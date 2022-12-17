use csv::Writer;
use std::io::Stdout;
use crate::daimojo_library::{MOJO_DataType, RawColumnBuffer, RawFrame, RawPipeline};

pub struct FrameExporter<'a> {
    saved_batches: usize,
    saved_rows: usize,
    wtr: Writer<Stdout>,
    ocols: Vec<RawColumnBuffer<'a>>,
}

impl<'a> FrameExporter<'a> {
    pub fn init(pipeline: &RawPipeline, frame: &'a RawFrame) -> std::io::Result<Self> {
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

    pub fn export_frame(&mut self, rows: usize) -> std::io::Result<()> {
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
