//! Convenient abstraction for daimojo interface

use std::ffi::{CStr, CString};
use std::io::{Error, ErrorKind};
use std::os::raw::c_char;
use std::rc::Rc;
use crate::daimojo_library::{DaiMojoLibrary, MOJO_Col, MOJO_DataType, MOJO_Frame, MOJO_Model, PArrayOperations, PCharArrayOperations};

pub mod daimojo_library;

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
        if self.lib.is_valid(mojo_model) == 0 {
            // license check or something else failed, see above stderr messages
            return Err(Error::new(ErrorKind::InvalidData, format!("Pipeline '{file}' is not valid")));
        }
        Ok(MojoPipeline { lib: self.lib.clone(), mojo_model, })
    }
}

impl Drop for DaiMojo {
    fn drop(&mut self) {
        log::trace!("Dropping library");
    }
}

pub struct MojoPipeline {
    lib: Rc<DaiMojoLibrary>,
    mojo_model: *const MOJO_Model,
}

impl MojoPipeline {

    pub fn inputs(&self) -> Vec<(String, MOJO_DataType)> {
        let count = self.lib.feature_num(self.mojo_model);
        Self::columns(count,
                      self.lib.feature_names(self.mojo_model).to_slice(count),
                      self.lib.feature_types(self.mojo_model).to_slice(count))
    }

    pub fn outputs(&self) -> Vec<(String, MOJO_DataType)> {
        let count = self.lib.output_num(self.mojo_model);
        Self::columns(count,
                      self.lib.output_names(self.mojo_model).to_slice(count),
                      self.lib.output_types(self.mojo_model).to_slice(count))
    }

    fn columns(cnt: usize, names: &[*const c_char], types: &[MOJO_DataType]) -> Vec<(String, MOJO_DataType)> {
        let mut result = Vec::new();
        for i in 0..cnt {
            let col_name = unsafe { CStr::from_ptr(names[i]) }.to_string_lossy().to_string();
            let col_type = types[i];
            result.push((col_name, col_type));
        }
        result
    }

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

    pub fn predict(&self, frame: &mut MojoFrame) {
        self.lib.predict(self.mojo_model, frame.mojo_frame);
    }

    pub fn uuid(&self) -> &str {
        self.lib.uuid(self.mojo_model).to_str()
            .expect("bad chars")
    }

    pub fn time_created(&self) -> u64 {
        self.lib.time_created(self.mojo_model)
    }

    pub fn missing_values(&self) -> Vec<String> {
        self.lib.missing_values(self.mojo_model)
            .to_vec_string(self.lib.missing_values_num(self.mojo_model))
    }
}

impl Drop for MojoPipeline {
    fn drop(&mut self) {
        log::trace!("Dropping pipeline [UUID={}]", self.uuid());
        self.lib.delete_model(self.mojo_model);
    }
}

pub struct MojoFrame {
    lib: Rc<DaiMojoLibrary>,
    mojo_frame: *const MOJO_Frame,
    row_count: usize,
}

impl MojoFrame {
    fn column(&mut self, col_name: &str) -> Option<*const MOJO_Col> {
        let name = CString::new(col_name).unwrap();
        let p = self.lib.get_col_by_name(self.mojo_frame, name.as_ptr());
        if p.is_null() {
            None
        } else {
            Some(p)
        }
    }

    pub fn input_mut(&mut self, col_name: &str) -> Option<*mut u8> {
        let col = self.column(col_name)?;
        Some(self.lib.data(col))
    }

    pub fn output(&mut self, col_name: &str) -> Option<*const u8> {
        let col = self.column(col_name)?;
        Some(self.lib.data(col))
    }

    pub fn input_f32_mut(&mut self, col_name: &str) -> Option<&mut [f32]> {
        let data = self.input_mut(col_name)?;
        Some(unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(data), self.row_count) })
    }

    pub fn output_f32(&mut self, col_name: &str) -> Option<&[f32]> {
        let data = self.output(col_name)?;
        Some(unsafe { std::slice::from_raw_parts(std::mem::transmute(data), self.row_count) })
    }

    pub fn ncol(&self) -> usize {
        self.lib.frame_ncol(self.mojo_frame)
    }
}

impl Drop for MojoFrame {
    fn drop(&mut self) {
        log::trace!("Dropping frame [@0x{:x}]", unsafe { std::mem::transmute::<*const MOJO_Frame, usize>(self.mojo_frame) });
        self.lib.delete_frame(self.mojo_frame);
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
        let pipeline = daimojo.pipeline("data/iris/pipeline.mojo")?;
        println!("Pipeline UUID: {}", pipeline.uuid());
        println!("Time created: {}", pipeline.time_created());
        let mut frame = pipeline.frame(1);
        // fill input columns
        frame.input_f32_mut("sepal_len").unwrap()[0] = 5.1;
        frame.input_f32_mut("sepal_wid").unwrap()[0] = 3.5;
        frame.input_f32_mut("petal_len").unwrap()[0] = 1.4;
        frame.input_f32_mut("petal_wid").unwrap()[0] = 0.2;
        log::trace!("ncol before predict: {}", frame.ncol());
        pipeline.predict(&mut frame);
        log::trace!("ncol after predict: {}", frame.ncol());
        // present output columns
        let setosa = frame.output_f32("class.Iris-setosa").unwrap()[0];
        let versicolor = frame.output_f32("class.Iris-versicolor").unwrap()[0];
        let virginica = frame.output_f32("class.Iris-virginica").unwrap()[0];
        println!("Result: {} {} {}", setosa, versicolor, virginica);
        assert_eq!(setosa, 0.43090245);
        assert_eq!(versicolor, 0.28463825583457947);
        assert_eq!(virginica, 0.28445929288864136);
        Ok(())
    }
}
