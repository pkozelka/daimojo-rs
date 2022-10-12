//! Convenient abstraction for daimojo interface

use std::ffi::{CStr, CString};
use std::rc::Rc;
use crate::daimojo_library::{MOJO_Frame, MOJO_Model};
use crate::DaiMojoLibrary;

pub struct DaiMojo {
    lib: Rc<DaiMojoLibrary>,
}

impl DaiMojo {
    pub fn library(libname: &str) -> std::io::Result<Self> {
        let lib = DaiMojoLibrary::open(libname)?;
        Ok(Self {lib: Rc::new(lib)})
    }

    pub fn version(&self) -> String {
        self.lib.version().to_string_lossy().to_string()
    }

    pub fn pipeline(&self, file: &str) -> std::io::Result<MojoPipeline> {
        let mojo_model = self.lib.new_model(&CString::new(file)?, &CString::new("")?);
        Ok(MojoPipeline { lib: self.lib.clone(), mojo_model, })
    }
}

pub struct MojoPipeline {
    lib: Rc<DaiMojoLibrary>,
    mojo_model: *const MOJO_Model,
}

impl MojoPipeline {
    /// This is a helper function that is not directly represented in the API
    pub fn frame(&self, row_count: usize) -> MojoFrame {
        // inputs
        // outputs
        // let mojo_frame = self.lib.new_frame();
        // MojoFrame { lib: self.lib.clone(), mojo_frame, row_count}
        todo!()
    }
}

pub struct MojoFrame {
    lib: Rc<DaiMojoLibrary>,
    mojo_frame: *const MOJO_Frame,
    row_count: usize,
}

#[cfg(test)]
mod tests {
    use super::DaiMojo;

    #[test]
    fn simple_test() -> std::io::Result<()>{
        let daimojo = DaiMojo::library("lib/linux_x64/libdaimojo.so")?;
        let version = daimojo.version();
        println!("Library version: {version}");
        let pipeline = daimojo.pipeline("../mojo2/data/iris/pipeline.mojo")?;
        let frame = pipeline.frame(1);
        // fill input columns
        frame.set_float32("sepal_len", &[5.1]);
        frame.set_float32("sepal_wid", &[3.5]);
        frame.set_float32("petal_len", &[1.4]);
        frame.set_float32("petal_wid", &[0.2]);
        pipeline.predict(frame);
        // present output columns
        let setosa = frame.get_float32("class.Iris-setosa");
        assert_eq!(setosa[0], 0.43090245);
        //TODO the rest
        Ok(())
    }
}
