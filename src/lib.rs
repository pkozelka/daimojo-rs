//! Convenient abstraction for daimojo interface

pub use csv_export::FrameExporter;
pub use csv_import::FrameImporter;
pub use daimojo_library::{DaiMojoLibrary, MOJO_DataType, MOJO_INT32_NAN, MOJO_INT64_NAN, MOJO_Transform_Ops};
pub use daimojo_library::{RawFrame, RawModel, RawPipeline};
pub use error::{MojoError, Result};

mod daimojo_library;
mod carray;
mod csv_import;
mod csv_export;
mod error;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::daimojo_library::{DaiMojoLibrary, MOJO_Transform_Ops, RawFrame, RawModel, RawPipeline};
    use crate::error;

    // const LIBDAIMOJO_SO: &str = "lib/linux_x64/libdaimojo.so";
    const LIBDAIMOJO_SO: &str = "libdaimojo.so";

    #[test]
    fn simple_iris_test() -> error::Result<()>{
        let lib = DaiMojoLibrary::load(Path::new(LIBDAIMOJO_SO))?;
        let version = lib.version();
        println!("Library version: {version}");
        let model = RawModel::load(&lib, "data/iris/pipeline.mojo", "")?;
        println!("Pipeline UUID: {}", model.uuid().to_string_lossy());
        println!("Time created: {}", model.time_created_utc());
        let pipeline = RawPipeline::new(&model, MOJO_Transform_Ops::PREDICT as MOJO_Transform_Ops)?;
        let mut frame = RawFrame::new(&pipeline, 1)?;
        // fill input columns
        frame.input_f32_mut(0, /*"sepal_len"*/)?[0] = 5.1;
        frame.input_f32_mut(1, /*"sepal_wid"*/)?[0] = 3.5;
        frame.input_f32_mut(2, /*"petal_len"*/)?[0] = 1.4;
        frame.input_f32_mut(3, /*"petal_wid"*/)?[0] = 0.2;
        log::trace!("ncol before predict: {}", frame.ncol());
        pipeline.transform(&mut frame, 1, false)?;
        log::trace!("ncol after predict: {}", frame.ncol());
        // present output columns
        let setosa     = frame.output_f32(0)?[0];
        let versicolor = frame.output_f32(1)?[0];
        let virginica  = frame.output_f32(2)?[0];
        println!("Result: {} {} {}", setosa, versicolor, virginica);
        assert_eq!(setosa, 0.43090245);
        assert_eq!(versicolor, 0.28463825583457947);
        assert_eq!(virginica, 0.28445929288864136);
        Ok(())
    }

    #[test]
    fn simple_wine_test() -> error::Result<()> {
        let lib = DaiMojoLibrary::load(LIBDAIMOJO_SO)?;
        let version = lib.version();
        println!("Library version: {version}");

        let model = RawModel::load(&lib, "data/wine/pipeline.mojo", "")?;
        println!("Pipeline UUID: {}", model.uuid().to_string_lossy());
        println!("Time created: {}", model.time_created_utc());
        let pipeline = RawPipeline::new(&model, MOJO_Transform_Ops::PREDICT as MOJO_Transform_Ops)?;
        let mut frame = RawFrame::new(&pipeline, 5)?;
        // fill input columns
        let fixed_acidity = frame.input_f32_mut(0/*"fixed acidity"*/).unwrap();
        fixed_acidity[0] = 11.8;
        fixed_acidity[1] = 7.2;
        fixed_acidity[2] = 6.4;
        fixed_acidity[3] = 8.6;
        fixed_acidity[4] = 7.3;
        // fixed_acidity.copy_from_slice(&[11.8, 7.2, 6.4, 8.6, 7.3]); // alternative way to fill it
        log::trace!("ncol before predict: {}", frame.ncol());
        pipeline.transform(&mut frame, 5, true).unwrap();
        log::trace!("ncol after predict: {}", frame.ncol());
        // present output columns
        let q3 = frame.output_f32(0/*"quality.3"*/).unwrap();
        println!("quality.3={q3:?}");
        println!("quality.4={:?}", frame.output_f32(1).unwrap());
        println!("quality.5={:?}", frame.output_f32(2).unwrap());
        println!("quality.6={:?}", frame.output_f32(3).unwrap());
        println!("quality.7={:?}", frame.output_f32(4).unwrap());
        let q8 = frame.output_f32(5).unwrap();
        println!("quality.8={q8:?}", );
        assert_eq!(q3, [0.004791974, 0.0038865132, 0.003860102, 0.0048087006, 0.0038697503]);
        assert_eq!(q8, [0.03322292, 0.034191407, 0.03479463, 0.03333889, 0.04129037]);
        Ok(())
    }
}
