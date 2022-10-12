//! Convenient abstraction for daimojo interface

use std::ffi::CString;
use std::rc::Rc;
use crate::daimojo_library::{DaiMojoLibrary, MOJO_Frame, MOJO_Model, PArrayOperations};

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
        let mut names = Vec::new();
        let mut cols = Vec::new();
        // inputs
        {
            let icnt = self.lib.feature_num(self.mojo_model);
            let inames = self.lib.feature_names(self.mojo_model).to_slice(icnt);
            let itypes = self.lib.feature_types(self.mojo_model).to_slice(icnt);
            for i in 0..icnt {
                names.push(inames[i]);
                cols.push(self.lib.new_col(itypes[i], row_count));
            }
        }
        // outputs
        {
            let ocnt = self.lib.output_num(self.mojo_model);
            let onames = self.lib.output_names(self.mojo_model).to_slice(ocnt);
            let otypes = self.lib.output_types(self.mojo_model).to_slice(ocnt);
            for i in 0..ocnt {
                names.push(onames[i]);
                cols.push(self.lib.new_col(otypes[i], row_count));
            }
        }
        // MojoFrame { lib: self.lib.clone(), mojo_frame, row_count}
        let mojo_frame = self.lib.new_frame(cols.as_ptr(), names.as_ptr(), names.len());
        MojoFrame { lib: self.lib.clone(), mojo_frame, row_count }
    }

    pub fn predict(&self, frame: &MojoFrame) {
        self.lib.predict(self.mojo_model, frame.mojo_frame);
    }
}

pub struct MojoFrame {
    lib: Rc<DaiMojoLibrary>,
    mojo_frame: *const MOJO_Frame,
    row_count: usize,
}

impl MojoFrame {
    pub fn data(&mut self, col_name: &str) -> &mut [f32] {
        let name = CString::new(col_name).unwrap();
        let col = self.lib.get_col_by_name(self.mojo_frame, name.as_ptr());
        let data = self.lib.data(col);
        unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(data), self.row_count) }
    }
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
        let mut frame = pipeline.frame(1);
        // fill input columns
        frame.data("sepal_len")[0] = 5.1;
        frame.data("sepal_wid")[0] = 3.5;
        frame.data("petal_len")[0] = 1.4;
        frame.data("petal_wid")[0] = 0.2;
        pipeline.predict(&frame);
        // present output columns
        let setosa = frame.data("class.Iris-setosa")[0];
        let versicolor = frame.data("class.Iris-versicolor")[0];
        let virginica = frame.data("class.Iris-virginica")[0];
        println!("Result: {} {} {}", setosa, versicolor, virginica);
        assert_eq!(setosa, 0.43090245);
        Ok(())
    }
}
