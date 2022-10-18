use std::collections::HashMap;
use std::io::ErrorKind;
use daimojo::daimojo_library::MOJO_DataType;
use daimojo::MojoPipeline;

const MOJO_I32_NAN: i32 = i32::MAX;
const MOJO_I64_NAN: i64 = i64::MAX;
const BATCH_SIZE: usize = 1000;

//TODO missing features:
// - processing in multiple batches
// - various CSV in/out flags
// - support variable column width
// - passing selected input columns to output
// - column name mapping + ignore case
// - headerless csv
// - set row size



pub fn cmd_predict(pipeline: &MojoPipeline, _output: Option<String>, input: Option<String>) -> std::io::Result<u8> {
    let mut frame = pipeline.frame(BATCH_SIZE);
    let mut rdr = csv::Reader::from_path(input.unwrap())?;
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
            let ptr = frame.input_mut(&col_name).unwrap();
            icols.push(ColumnData { data_type, data: ColumnArray { ptr }, csv_index})
        }
    }

    // read csv
    let mut total_rows = 0;
    for record in rdr.records() {
        let record = record?;
        // fill mojo row
        for col in &mut icols {
            let value = record.get(col.csv_index).expect("column disappeared?");
            col.item_from_str(total_rows, value);
        }
        total_rows += 1;
        //TODO: support multiple batches
        if total_rows == BATCH_SIZE { panic!("Batch size exceeded")}
    }

    // predict
    pipeline.predict(&mut frame);

    // output csv
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    let mut ocols = Vec::new();
    for (col_name, data_type) in pipeline.outputs() {
        wtr.write_field(&col_name)?;
        let ptr = frame.output(&col_name).unwrap();
        ocols.push(ColumnData { data_type, data: ColumnArray { ptr }, csv_index: 0});
    }
    wtr.write_record(None::<&[u8]>)?;

    for row in 0..total_rows {
        for col in &mut ocols {
            let s = col.item_to_string(row);
            wtr.write_field(s)?;
        }
        wtr.write_record(None::<&[u8]>)?;
    }
    Ok(0)
}

struct ColumnData<'a> {
    data_type: MOJO_DataType,
    data: ColumnArray<'a, 10000>,
    // data: *mut u8,
    csv_index: usize,
}

//TODO let's find something better than enum
#[repr(C)]
union ColumnArray<'a, const N: usize> {
    ptr: *const u8,
    f32_array: &'a mut [f32; N],
    f64_array: &'a mut [f64; N],
    i32_array: &'a mut [i32; N],
    i64_array: &'a mut [i64; N],
    str_array: &'a mut [*mut u8; N],
}

impl <'a> ColumnData<'a>  {
    fn item_from_str(&mut self, index: usize, value: &str) {
        match self.data_type {
            MOJO_DataType::MOJO_FLOAT => {
                let value = value.parse::<f32>().unwrap_or(f32::NAN);
                unsafe { self.data.f32_array[index] = value };
            }
            MOJO_DataType::MOJO_DOUBLE => {
                let value = value.parse::<f64>().unwrap_or(f64::NAN);
                unsafe { self.data.f64_array[index] = value };
            }
            MOJO_DataType::MOJO_INT32 => {
                let value = value.parse::<i32>().unwrap_or(MOJO_I32_NAN);
                unsafe { self.data.i32_array[index] = value };
            }
            MOJO_DataType::MOJO_INT64 => {
                let value = value.parse::<i64>().unwrap_or(MOJO_I64_NAN);
                unsafe { self.data.i64_array[index] = value };
            }
            MOJO_DataType::MOJO_STRING => {
                todo!()
            }
            MOJO_DataType::MOJO_UNKNOWN => panic!("unsupported column type")
        }
    }

    fn item_to_string(&self, index: usize) -> String {
        match self.data_type {
            MOJO_DataType::MOJO_FLOAT => {
                let value = unsafe { self.data.f32_array[index] };
                format!("{value}")
            }
            //TODO!
            _ => panic!("unsupported column type")
        }
    }
}
