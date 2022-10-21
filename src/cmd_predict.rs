use std::collections::HashMap;
use std::io::{ErrorKind, Stdout};
use std::mem::transmute;
use csv::Writer;
use daimojo::daimojo_library::MOJO_DataType;
use daimojo::{MojoFrame, MojoPipeline};

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

pub fn cmd_predict(pipeline: &MojoPipeline, _output: Option<String>, input: Option<String>, batch_size: usize) -> std::io::Result<u8> {
    let batch_size = batch_size_magic(&input, batch_size)?;

    let mut frame = pipeline.create_frame(batch_size);

    let mut rdr = csv::Reader::from_path(input.unwrap())?;
    let mut importer = FrameImporter::init(&pipeline, &mut frame, &mut rdr)?;
    let mut exporter = FrameExporter::init(&pipeline, &frame)?;
    // read csv
    let mut rdr_iter = rdr.records();
    while ! importer.eof {
        let rows = importer.import_frame(&mut rdr_iter)?;

        // predict
        pipeline.predict(&mut frame, rows);
        log::debug!("-- batch {rows} rows");

        // output csv
        exporter.export_frame(rows)?;
    }
    log::info!("Total rows: {}", exporter.saved_rows);
    //
    Ok(0)
}

struct FrameImporter {
    icols: Vec<(ColumnData, usize)>,
    batch_size: usize,
    eof: bool,
}

impl FrameImporter {
    pub fn init(pipeline: &MojoPipeline, frame: &mut MojoFrame, rdr: &mut csv::Reader<std::fs::File>) -> std::io::Result<Self> {
        let csv_headers = match rdr.headers() {
            Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Cannot read header: {e}"))),
            Ok(headers) => headers,
        };
        let csv_headers = csv_headers.iter().enumerate()
            .map(|(csv_index, col_name)| (col_name, csv_index))
            .collect::<HashMap<&str, usize>>();
        let mut icols = Vec::new();
        for (col_name, data_type) in pipeline.inputs() {
            if let Some(&csv_index) = csv_headers.get(col_name.as_str()) {
                let ptr = frame.input_mut(&col_name).expect(&format!("No buffer for input column '{col_name}'"));
                icols.push((ColumnData { data_type, array_start: ptr, current: ptr }, csv_index))
            }
        }
        Ok(Self {
            icols,
            batch_size: frame.nrow(),
            eof: rdr.is_done()
        })
    }

    pub fn import_frame(&mut self, rdr_iter: &mut csv::StringRecordsIter<std::fs::File>) -> std::io::Result<usize> {
        let mut rows = 0;
        if self.eof {
            return Ok(0);
        }
        ColumnData::reset_current_tuple(&mut self.icols);
        for record in rdr_iter {
            let record = record?;
            // fill mojo row
            for (col, csv_index) in &mut self.icols {
                let value = record.get(*csv_index).expect("column disappeared?");
                col.item_from_str(value);
            }
            rows += 1;
            if rows == self.batch_size {
                return Ok(rows)
            }
        }
        // ending prematurely => last batch
        self.eof = true;
        Ok(rows)
    }
}

struct FrameExporter {
    saved_batches: usize,
    saved_rows: usize,
    wtr: Writer<Stdout>,
    ocols: Vec<ColumnData>,
}

impl FrameExporter {
    fn init(pipeline: &MojoPipeline, frame: &MojoFrame) -> std::io::Result<Self> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        let mut ocols = Vec::new();
        for (col_name, data_type) in pipeline.outputs() {
            wtr.write_field(&col_name)?;
            let ptr = frame.output(&col_name).expect(&format!("No buffer for output column '{col_name}'"));
            ocols.push(ColumnData { data_type, array_start: ptr, current: ptr as *mut u8 });
        }
        wtr.write_record(None::<&[u8]>)?;
        wtr.flush()?;
        Ok(Self { saved_batches: 0, saved_rows:0, wtr, ocols})
    }

    fn export_frame(&mut self, rows: usize) -> std::io::Result<()> {
        ColumnData::reset_current(&mut self.ocols);
        for _ in 0..rows {
            for col in &mut self.ocols {
                let s = col.item_to_string();
                self.wtr.write_field(s)?;
            }
            self.wtr.write_record(None::<&[u8]>)?;
            self.wtr.flush()?;
        }
        self.saved_batches += 1;
        self.saved_rows += rows;
        Ok(())
    }
}

/// Read value from array at provided pointer, and move the pointer to the next item
fn unchecked_read_next<T: Copy>(ptr: &mut *mut/*const*/ u8) -> T {
    unsafe {
        let p: *const T = transmute(*ptr as usize);
        *ptr = p.offset(1) as *mut/*const*/ u8;
        p.read()
    }
}

/// Write value to array at provided pointer, and move the pointer to the next item
fn unchecked_write_next<T: Copy>(ptr: &mut *mut u8, value: T) {
    unsafe {
        let p: *mut T = transmute(*ptr as usize);
        *ptr = p.offset(1) as *mut u8;
        p.write(value)
    }
}

struct ColumnData {
    data_type: MOJO_DataType,
    array_start: *const u8,
    current: *mut u8,
}

impl ColumnData  {
    fn reset_current(vec: &mut Vec<Self>) {
        vec.iter_mut().for_each(|col| col.current = col.array_start as *mut u8);
    }

    fn reset_current_tuple<T>(vec: &mut Vec<(Self,T)>) {
        vec.iter_mut().for_each(|(col,_)| col.current = col.array_start as *mut u8);
    }

    fn item_from_str(&mut self, value: &str) {
        log::trace!("memset:{:?}:[@0x{:x}] = '{value}'",self.data_type, self.current as usize);
        match self.data_type {
            MOJO_DataType::MOJO_FLOAT => {
                let value = value.parse::<f32>().unwrap_or(f32::NAN);
                unchecked_write_next(&mut self.current, value);
            }
            MOJO_DataType::MOJO_DOUBLE => {
                let value = value.parse::<f64>().unwrap_or(f64::NAN);
                unchecked_write_next(&mut self.current, value);
            }
            MOJO_DataType::MOJO_INT32 => {
                let value = value.parse::<i32>().unwrap_or(MOJO_I32_NAN);
                unchecked_write_next(&mut self.current, value);
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = value.parse::<i64>().unwrap_or(MOJO_I64_NAN);
                unchecked_write_next(&mut self.current, value);
            }
            MOJO_DataType::MOJO_STRING => {
                todo!()
            }
            MOJO_DataType::MOJO_UNKNOWN => panic!("unsupported column type")
        }
    }

    fn item_to_string(&mut self) -> String {
        match self.data_type {
            MOJO_DataType::MOJO_FLOAT => {
                let value = unchecked_read_next::<f32>(&mut self.current);
                format!("{value}")
            }
            MOJO_DataType::MOJO_DOUBLE => {
                let value = unchecked_read_next::<f64>(&mut self.current);
                format!("{value}")
            }
            MOJO_DataType::MOJO_INT32 => {
                let value = unchecked_read_next::<i32>(&mut self.current);
                format!("{value}")
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = unchecked_read_next::<i64>(&mut self.current);
                format!("{value}")
            }
            //TODO!
            _ => panic!("unsupported column type")
        }
    }
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
