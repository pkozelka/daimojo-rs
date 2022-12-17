use std::collections::HashMap;
use std::io::{ErrorKind, Stdout};
use csv::Writer;
use daimojo::daimojo_library::{MOJO_DataType, RawColumnBuffer, RawFrame, RawPipeline};

const MOJO_I32_NAN: i32 = i32::MAX;
const MOJO_I64_NAN: i64 = i64::MAX;

//TODO missing features:
// - processing in multiple batches
// - various CSV in/out flags
// - support variable column width
// - passing selected input columns to output
// - column name mapping + ignore case
// - headerless csv
// - set row size

/// Minimum size returned by [batch_size_magic].
const MIN_BATCH_SIZE: usize = 1000;

pub fn cmd_predict(pipeline: &RawPipeline, _output: Option<String>, input: Option<String>, batch_size: usize) -> anyhow::Result<u8> {
    let batch_size = batch_size_magic(&input, batch_size)?;

    let frame = RawFrame::new(pipeline, batch_size)?;

    let mut rdr = csv::Reader::from_path(input.unwrap())?;
    let mut importer = FrameImporter::init(&pipeline, &frame, &mut rdr)?;
    let mut exporter = FrameExporter::init(&pipeline, &frame)?;
    // read csv
    let mut rdr_iter = rdr.records();
    while ! importer.eof {
        let rows = importer.import_frame(&mut rdr_iter)?;

        // predict
        pipeline.transform(&frame, rows, false)?;
        log::debug!("-- batch {rows} rows");

        // output csv
        exporter.export_frame(rows)?;
    }
    log::info!("Total rows: {}", exporter.saved_rows);
    //
    Ok(0)
}

struct FrameImporter<'a> {
    icols: Vec<RawColumnBuffer<'a>>,
    csv_indices: Vec<usize>,
    batch_size: usize,
    eof: bool,
}

impl<'a> FrameImporter<'a> {
    pub fn init(pipeline: &RawPipeline, frame: &'a RawFrame, rdr: &mut csv::Reader<std::fs::File>) -> std::io::Result<Self> {
        let model = pipeline.model;
        let csv_headers = match rdr.headers() {
            Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Cannot read header: {e}"))),
            Ok(headers) => headers,
        };
        let csv_headers = csv_headers.iter().enumerate()
            .map(|(csv_index, col_name)| (col_name, csv_index))
            .collect::<HashMap<&str, usize>>();
        let mut icols = Vec::new();
        let mut csv_indices = Vec::new();
        for (index, col) in model.feature_names().enumerate() {
            if let Some(&csv_index) = csv_headers.get(col.as_ref()) {
                // println!("Rust: input_data({index}='{}') -> {:X}", col.name, ptr as usize);
                icols.push(frame.input_col(index));
                csv_indices.push(csv_index);
            }
        }
        Ok(Self {
            icols,
            csv_indices,
            batch_size: frame.nrow,
            eof: rdr.is_done()
        })
    }

    pub fn import_frame(&mut self, rdr_iter: &mut csv::StringRecordsIter<std::fs::File>) -> std::io::Result<usize> {
        let mut row = 0;
        if self.eof {
            return Ok(0);
        }
        RawColumnBuffer::reset_current(&mut self.icols);
        for record in rdr_iter {
            let record = record?;
            // fill mojo row
            for (feature_index, col) in &mut self.icols.iter_mut().enumerate() {
                let csv_index = self.csv_indices[feature_index];
                let value = record.get(csv_index).expect("column disappeared?");
                Self::item_from_str(row, col, value);
            }
            row += 1;
            if row == self.batch_size {
                return Ok(row)
            }
        }
        // ending prematurely => last batch
        self.eof = true;
        Ok(row)
    }

    fn item_from_str(row: usize, col: &mut RawColumnBuffer, value: &str) {
        // log::trace!("memset:{:?}:[@0x{:x}] = '{value}'", col.data_type, col.current as usize);
        match col.data_type {
            MOJO_DataType::MOJO_BOOL => {
                let value = mojo2_parse_bool(value);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_FLOAT => {
                let value = value.parse::<f32>().unwrap_or(f32::NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_DOUBLE => {
                let value = value.parse::<f64>().unwrap_or(f64::NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_INT32 => {
                let value = value.parse::<i32>().unwrap_or(MOJO_I32_NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = value.parse::<i64>().unwrap_or(MOJO_I64_NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_STRING => {
                col.unchecked_write_str(row, value);
            }
            MOJO_DataType::MOJO_UNKNOWN => panic!("unsupported column type")
        }
    }
}

struct FrameExporter<'a> {
    saved_batches: usize,
    saved_rows: usize,
    wtr: Writer<Stdout>,
    ocols: Vec<RawColumnBuffer<'a>>,
}

impl<'a> FrameExporter<'a> {
    fn init(pipeline: &RawPipeline, frame: &'a RawFrame) -> std::io::Result<Self> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        let mut ocols = Vec::new();
        for (index, name) in pipeline.output_names().enumerate() {
            wtr.write_field(&name.as_ref())?;
            // println!("Rust: output_data({index}='{}') -> {:X}", col.name, ptr as usize);
            ocols.push(frame.output_col(index));
        }
        wtr.write_record(None::<&[u8]>)?;
        wtr.flush()?;
        Ok(Self { saved_batches: 0, saved_rows:0, wtr, ocols})
    }

    fn export_frame(&mut self, rows: usize) -> std::io::Result<()> {
        RawColumnBuffer::reset_current(&mut self.ocols);
        for row in 0..rows {
            for col in &mut self.ocols {
                let s = Self::item_to_string(row, col);
                self.wtr.write_field(s)?;
            }
            self.wtr.write_record(None::<&[u8]>)?;
            self.wtr.flush()?;
        }
        self.saved_batches += 1;
        self.saved_rows += rows;
        Ok(())
    }

    fn item_to_string(row: usize, col: &mut RawColumnBuffer) -> String {
        match col.data_type {
            MOJO_DataType::MOJO_BOOL => {
                let value = col.unchecked_read_next::<bool>();
                format!("{value}")
            }
            MOJO_DataType::MOJO_FLOAT => {
                let value = col.unchecked_read_next::<f32>();
                format!("{value}")
            }
            MOJO_DataType::MOJO_DOUBLE => {
                let value = col.unchecked_read_next::<f64>();
                format!("{value}")
            }
            MOJO_DataType::MOJO_INT32 => {
                let value = col.unchecked_read_next::<i32>();
                format!("{value}")
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = col.unchecked_read_next::<i64>();
                format!("{value}")
            }
            MOJO_DataType::MOJO_STRING => {
                col.unchecked_read_string(row).to_string()
            }
            MOJO_DataType::MOJO_UNKNOWN => panic!("unsupported column type")
        }
    }
}

fn mojo2_parse_bool(s: &str) -> bool {
    const VALUES: &[&str] = &[
        "true", "True", "TRUE", "1", "1.0",
        "false", "False", "FALSE", "0", "0.0"];
    let mut result = true;
    for &item in VALUES {
        if item == "false" {
            result = false;
        }
        if item == s {
            return result;
        }
    }
    log::trace!("Invalid bool value: '{s}'");
    false
}

/// Heuristics to estimate best batch size possible for given input.
/// The goal is, try to hold everything in memory, but don't request too much of it.
fn batch_size_magic(input: &Option<String>, batch_size: usize) -> std::io::Result<usize> {
    Ok(match (batch_size, &input) {
        (0, None) => MIN_BATCH_SIZE * 10,
        (0, Some(path)) => {
            let input_len = std::fs::metadata(path)?.len();
            let mut batch_size = input_len / 50;
            if batch_size < 1000 {
                batch_size = 1000;
            }
            log::warn!("Batch size was automatically set to {batch_size}");
            batch_size as usize
        }
        (batch_size, _) => batch_size,
    })
}

#[cfg(test)]
mod tests {
    use crate::cmd_predict::{unchecked_read_next, unchecked_write_next};

    #[test]
    fn test_readptr() {
        let a: [f32;4] = [7.0,11.0,13.0,17.0];
        let mut p= a.as_ptr() as *mut u8;
        let next = unchecked_read_next::<f32>(&mut p);
        println!("A[0]: {}", next);
        assert_eq!(7.0, next);
        let next = unchecked_read_next::<f32>(&mut p);
        println!("A[1]: {}", next);
        assert_eq!(11.0, next);
        let next = unchecked_read_next::<f32>(&mut p);
        println!("A[2]: {}", next);
        assert_eq!(13.0, next);
        let next = unchecked_read_next::<f32>(&mut p);
        println!("A[3]: {}", next);
        assert_eq!(17.0, next);
    }

    #[test]
    fn test_writeptr() {
        let a: [f32;4] = [7.0,11.0,13.0,17.0];
        let mut p= a.as_ptr() as *mut u8;
        unchecked_write_next::<f32>(&mut p, 1.2);
        unchecked_write_next::<f32>(&mut p, 3.4);
        unchecked_write_next::<f32>(&mut p, 5.6);
        unchecked_write_next::<f32>(&mut p, 7.8);
        let s = a.iter().map(|v|format!("{v}")).collect::<Vec<String>>();
        let s = s.join(",");
        println!("{s}");
        assert_eq!(1.2,a[0]);
        assert_eq!(7.8,a[3]);
    }

}
