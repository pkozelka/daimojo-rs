use std::io::ErrorKind;
use std::collections::HashMap;
use crate::daimojo_library::{MOJO_DataType, MOJO_INT32_NAN, MOJO_INT64_NAN, RawColumnBuffer, RawFrame, RawPipeline};
use crate::{error, MojoError};

pub struct FrameImporter<'a> {
    icols: Vec<RawColumnBuffer<'a>>,
    csv_indices: Vec<usize>,
    batch_size: usize,
    eof: bool,
}

impl<'a> FrameImporter<'a> {
    pub fn init(pipeline: &RawPipeline, frame: &'a RawFrame, rdr: &mut csv::Reader<std::fs::File>) -> error::Result<Self> {
        let model = pipeline.model;
        let csv_headers = match rdr.byte_headers() {
            Err(e) => Err(std::io::Error::new(ErrorKind::InvalidData, format!("Cannot read header: {e}")))?,
            Ok(headers) => headers,
        };
        let csv_headers: HashMap<&[u8], usize> = csv_headers.iter().enumerate()
            .map(|(csv_index, col_name)| (col_name, csv_index))
            .collect();
        let mut icols = Vec::new();
        let mut csv_indices = Vec::new();
        let mut missing_data = None;
        for (index, name) in model.feature_names_iter().enumerate() {
            if let Some(&csv_index) = csv_headers.get(name.to_bytes()) {
                // println!("Rust: input_data({index}='{}') -> {:X}", col.name, ptr as usize);
                icols.push(frame.input_col(index)?);
                csv_indices.push(csv_index);
            } else {
                log::error!("Unknown input column name: {}", name.to_string_lossy());
                missing_data = Some(index);
            }
        }
        if let Some(index) = missing_data {
            return Err(MojoError::InvalidInputIndex(index));
        }
        Ok(Self {
            icols,
            csv_indices,
            batch_size: frame.nrow,
            eof: rdr.is_done()
        })
    }

    pub fn import_frame(&mut self, rdr_iter: &mut csv::StringRecordsIter<std::fs::File>) -> std::io::Result<Option<usize>> {
        let mut row = 0;
        if self.eof {
            return Ok(None);
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
                return Ok(Some(row))
            }
        }
        // ending prematurely => last batch
        self.eof = true;
        Ok(if row == 0 { None } else { Some(row) })
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
                //TODO: if the string has decimal places, parse it and change to int
                let value = value.parse::<i32>().unwrap_or(MOJO_INT32_NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = value.parse::<i64>().unwrap_or(MOJO_INT64_NAN);
                col.unchecked_write_next(value);
            }
            MOJO_DataType::MOJO_STRING => {
                col.unchecked_write_str(row, value);
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
