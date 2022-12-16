//! Raw API implementation for interface of shared library "daimojo"
//!
#![allow(non_snake_case)]

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::io::ErrorKind;
use std::mem::transmute;
use std::os::raw::c_char;
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use dlopen2::wrapper::{Container, WrapperApi};

use crate::error;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct MOJO_Model {
    // capabilities: MOJO_Transform_Flags_Type,
    is_valid: bool,
    uuid: *const c_char,
    time_created: u64,
    missing_values_count: usize,
    missing_values: *const *const c_char,
    feature_count: usize,
    feature_names: *const *const c_char,
    feature_types: *const MOJO_DataType,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct MOJO_Pipeline {
    model: *const MOJO_Model,
    flags: MOJO_Transform_Flags_Type,
    output_count: usize,
    // output_names: *const *const c_char,
    // output_types: *const MOJO_DataType,
}

#[allow(non_camel_case_types)]
pub struct MOJO_Frame {}

#[allow(non_camel_case_types)]
pub struct MOJO_Col {}

/// An alias for array of C strings (`char **`)
pub type PCharArray = *const *const c_char;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy,Clone,Debug)]
pub enum MOJO_DataType {
    MOJO_UNKNOWN = 0,
    /// [i8] byte-represented boolean, 0=false, 1=true, NA is not defined
    MOJO_BOOL = 1,
    /// [i32] 4 bytes signed integer
    MOJO_INT32 = 2,
    /// [i64] 8 bytes signed integer
    MOJO_INT64 = 3,
    /// [f32] 4 bytes floating point
    MOJO_FLOAT = 4,
    /// [f64] 8 bytes floating point
    MOJO_DOUBLE = 5,
    /// c++ std::string, NA=empty string
    MOJO_STRING = 6,
}

#[allow(non_camel_case_types)]
pub type MOJO_Transform_Flags_Type = u32;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy,Clone,Debug)]
pub enum MOJO_Transform_Flags {
    /// normal prediction
    PREDICT = 1 << 0,
    /// prediction interval
    INTERVAL = 1 << 1,
    /// SHAP values
    CONTRIBS_RAW = 1 << 2,
    /// SHAP values mapped to original features
    CONTRIBS_ORIGINAL = 1 << 3,
}

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoVersionBindings {
    #[dlopen2_name = "MOJO_Version"]
    mojo_version: unsafe extern "C" fn() -> *const c_char,
}

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoBindings {
    // Model
    MOJO_NewModel: unsafe extern "C" fn(filename: *const c_char, tf_lib_prefix: *const c_char) -> *const MOJO_Model,
    MOJO_DeleteModel: unsafe extern "C" fn(mojo_model: *const MOJO_Model),
    // Pipeline
    MOJO_NewPipeline: unsafe extern "C" fn(mojo_model: *const MOJO_Model, flags: MOJO_Transform_Flags_Type) -> *const MOJO_Pipeline,
    MOJO_DeletePipeline: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline),
    MOJO_Transform: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, frame: *const MOJO_Frame, nrow: usize, debug: bool),
    /*?for now?*/
    MOJO_Output_Name: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, index: usize) -> *const c_char,
    /*?for now?*/
    MOJO_Output_Type: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, index: usize) -> MOJO_DataType,
    // Frame
    MOJO_Pipeline_NewFrame: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, nrow: usize) -> *const MOJO_Frame,
    MOJO_DeleteFrame: unsafe extern "C" fn(frame: *const MOJO_Frame),
    MOJO_FrameNcol: unsafe extern "C" fn(frame: *const MOJO_Frame) -> usize,
    MOJO_Input_Data: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, frame: *const MOJO_Frame, index: usize) -> *mut u8,
    MOJO_Output_Data: unsafe extern "C" fn(pipeline: *const MOJO_Pipeline, frame: *const MOJO_Frame, index: usize) -> *const u8,
    // String values support
    MOJO_Column_Write_Str: unsafe extern "C" fn(buffer: *mut u8, index: usize, value: *const c_char),
    MOJO_Column_Read_Str: unsafe extern "C" fn(buffer: *const u8, index: usize) -> *const c_char,
}

pub struct DaiMojoLibrary {
    api: Container<DaiMojoBindings>,
    version: String,
}

impl DaiMojoLibrary {

    pub fn load<P: AsRef<Path>>(libfile: P) -> error::Result<Self> {
        let libfile = libfile.as_ref().canonicalize()?;
        let libfile = libfile.to_str().expect(&format!("Not a valid unicode pathname: {}", libfile.to_string_lossy()));
        let version_api: Container<DaiMojoVersionBindings> = unsafe { Container::load(libfile) }
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{libfile}: {e:?}")))?;
        let version = unsafe { CStr::from_ptr(version_api.mojo_version()) }.to_string_lossy();
        log::debug!("Version: {version}");

        if !version.starts_with("2.") {
            return Err(error::MojoError::UnsupportedApi(libfile.to_string(), version.to_string()));
        }
        // TODO: isn't there a way to avoid loading again?
        let container = unsafe { Container::load(libfile) };
        let api = container?;
        Ok(Self { api, version: version.to_string()})
    }

    pub fn version(&self) -> Cow<str> {
        Cow::from(&self.version)
    }
}

pub struct RawColumnMeta<'a> {
    pub name: Cow<'a, str>,
    pub column_type: MOJO_DataType,
}

pub struct RawModel<'a> {
    lib: &'a DaiMojoLibrary,
    model_ptr: *const MOJO_Model,
}

impl<'a> RawModel<'a> {
    pub fn load<P: AsRef<Path>>(lib: &'a DaiMojoLibrary, filename: P, tf_lib_prefix: &str) -> std::io::Result<Self> {
        let filename = filename.as_ref().canonicalize()?;
        let filename = filename.to_str().expect(&format!("Not a valid unicode pathname: {}", filename.to_string_lossy()));
        let filename = CString::new(filename)?;
        let tf_lib_prefix = CString::new(tf_lib_prefix)?;
        let model_ptr = unsafe {
            lib.api.MOJO_NewModel(filename.as_ptr(), tf_lib_prefix.as_ptr())
        };
        if model_ptr.is_null() {
            return Err(std::io::Error::new(ErrorKind::NotFound, format!("File not found: {}", filename.to_string_lossy())));
        }
        Ok(Self { lib, model_ptr, })
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        unsafe { (*self.model_ptr).is_valid }
    }

    pub fn uuid(&self) -> Cow<str> {
        unsafe { CStr::from_ptr((*self.model_ptr).uuid) }.to_string_lossy()
    }

    pub fn time_created_utc(&self) -> DateTime<Utc> {
        let time_created = unsafe { (*self.model_ptr).time_created };
        let n = NaiveDateTime::from_timestamp_opt(time_created as i64, 0).unwrap();
        DateTime::from_utc(n, Utc)
    }

    pub fn missing_values(&self) -> Vec<Cow<str>> {
        unsafe {
            let count = (*self.model_ptr).missing_values_count;
            let mut vec = Vec::with_capacity(count);
            let mut p = (*self.model_ptr).missing_values;
            for _ in 0..count {
                let s = CStr::from_ptr(p.read()).to_string_lossy();
                p = p.add(1);
                vec.push(s);
            }
            vec
        }
    }

    pub fn features(&self) -> Vec<RawColumnMeta<'a>> {
        unsafe {
            let count = (*self.model_ptr).feature_count;
            let pname = (*self.model_ptr).feature_names;
            let ptype = (*self.model_ptr).feature_types;
            columns_from(count, pname, ptype)
        }
    }
}

impl <'a> Drop for RawModel<'a> {
    fn drop(&mut self) {
        log::trace!("calling MOJO_DeleteModel()");
        unsafe { self.lib.api.MOJO_DeleteModel(self.model_ptr) }
    }
}

pub struct RawPipeline<'a> {
    lib: &'a DaiMojoLibrary,
    pipeline_ptr: *const MOJO_Pipeline,
    pub model: &'a RawModel<'a>,
}

impl<'a> RawPipeline<'a> {
    pub fn new(model: &'a RawModel, flags: MOJO_Transform_Flags_Type) -> error::Result<Self> {
        let pipeline_ptr = unsafe { model.lib.api.MOJO_NewPipeline(model.model_ptr, flags)};
        Ok(Self {
            lib: model.lib,
            pipeline_ptr,
            model,
        })
    }

    pub fn outputs(&self) -> Vec<RawColumnMeta<'a>> {
        unsafe {
            let count = (*self.pipeline_ptr).output_count;
            let mut columns = Vec::with_capacity(count);
            for i in 0..count {
                let cname = self.lib.api.MOJO_Output_Name(self.pipeline_ptr, i);
                let cname = CStr::from_ptr(cname).to_string_lossy();
                let ctype = self.lib.api.MOJO_Output_Type(self.pipeline_ptr, i);
                columns.push(RawColumnMeta { name: cname, column_type: ctype });
            }
            columns
        }
    }

    pub fn transform(&self, frame: &RawFrame, nrow: usize, debug: bool) -> error::Result<()> {
        unsafe {
            self.lib.api.MOJO_Transform(self.pipeline_ptr, frame.frame_ptr, nrow, debug);
        }
        Ok(())
    }
}

impl<'a> Drop for RawPipeline<'a> {
    fn drop(&mut self) {
        unsafe { self.lib.api.MOJO_DeletePipeline(self.pipeline_ptr); }
    }
}

pub struct RawFrame<'a> {
    lib: &'a DaiMojoLibrary,
    frame_ptr: *const MOJO_Frame,
    pub nrow: usize,
    pipeline_ptr: *const MOJO_Pipeline,
}

impl<'a> RawFrame<'a> {
    pub fn new(pipeline: &'a RawPipeline, nrow: usize) -> error::Result<RawFrame<'a>> {
        let pipeline_ptr = pipeline.pipeline_ptr;
        let frame_ptr = unsafe { pipeline.lib.api.MOJO_Pipeline_NewFrame(pipeline_ptr, nrow) };
        Ok(Self {
            lib: pipeline.lib,
            frame_ptr,
            nrow,
            pipeline_ptr,
        })
    }

    pub fn ncol(&self) -> usize {
        unsafe { self.lib.api.MOJO_FrameNcol(self.frame_ptr) }
    }

    pub unsafe fn input_data(&self, index: usize) -> *mut u8 {
        unsafe {
            self.lib.api.MOJO_Input_Data(self.pipeline_ptr, self.frame_ptr, index)
        }
    }

    pub fn input_col(&self, feature_index: usize) -> RawColumnBuffer {
        let ptr = unsafe { self.input_data(feature_index) }; //.expect(&format!("No buffer for input column '{}'", col.name));
        let data_type = unsafe {
            let model = (*self.pipeline_ptr).model;
            (*model).feature_types.offset(feature_index as isize).read()
        };
        RawColumnBuffer::new(self.lib, data_type, ptr)
    }

    pub unsafe fn output_data(&self, index: usize) -> *const u8 {
        unsafe {
            self.lib.api.MOJO_Output_Data(self.pipeline_ptr, self.frame_ptr, index)
        }
    }

    pub fn output_col(&self, output_index: usize) -> RawColumnBuffer {
        let ptr = unsafe { self.output_data(output_index) }; //.expect(&format!("No buffer for input column '{}'", col.name));
        let data_type = unsafe {
            self.lib.api.MOJO_Output_Type(self.pipeline_ptr, output_index)
            // (*self.pipeline_ptr).output_types.offset(output_index as isize).read()
        };
        RawColumnBuffer::new(self.lib, data_type, ptr)
    }

    pub fn input_f32_mut(&mut self, index: usize) -> Option<&mut [f32]> {
        Some(unsafe {
            let data = self.input_data(index);
            std::slice::from_raw_parts_mut(transmute(data), self.nrow)
        })
    }

    pub fn output_f32(&self, index: usize) -> Option<&[f32]> {
        Some(unsafe {
            let data = self.output_data(index);
            std::slice::from_raw_parts(transmute(data), self.nrow)
        })
    }
}

impl<'a> Drop for RawFrame<'a> {
    fn drop(&mut self) {
        unsafe { self.lib.api.MOJO_DeleteFrame(self.frame_ptr) };
    }
}

pub struct RawColumnBuffer<'a> {
    lib: &'a DaiMojoLibrary,
    pub data_type: MOJO_DataType,
    array_start: *const u8,
    current: *mut u8,
}

impl<'a> RawColumnBuffer<'a> {

    fn new(lib: &'a DaiMojoLibrary, data_type: MOJO_DataType, ptr: *const u8) -> Self {
        Self {
            lib,
            data_type,
            array_start: ptr,
            current: ptr as *mut u8
        }
    }

    pub fn reset_current(vec: &mut Vec<Self>) {
        vec.iter_mut().for_each(|col| col.current = col.array_start as *mut u8);
    }

    /// Write value to array at provided pointer, and move the pointer to the next item
    pub fn unchecked_write_next<T: Copy>(&mut self, value: T) {
        unsafe {
            let p: *mut T = transmute(self.current as usize);
            self.current = p.offset(1) as *mut u8;
            p.write(value)
        }
    }

    /// Read value from array at provided pointer, and move the pointer to the next item
    pub fn unchecked_read_next<T: Copy>(&mut self) -> T {
        unsafe {
            let p: *const T = transmute(self.current as usize);
            self.current = p.offset(1) as *mut u8;
            p.read()
        }
    }

    pub fn unchecked_write_str(&mut self, row: usize, value: &str) {
        unsafe {
            let value = CString::from_vec_unchecked(value.as_bytes().to_vec());
            self.lib.api.MOJO_Column_Write_Str(self.array_start as *mut u8, row, value.as_ptr());
        }
    }

    pub fn unchecked_read_string(&mut self, row: usize) -> Cow<str>{
        unsafe {
            let value = self.lib.api.MOJO_Column_Read_Str(self.array_start as *mut u8, row);
            CStr::from_ptr(value).to_string_lossy()
        }
    }
}


fn columns_from<'a>(count: usize, mut pname: *const *const c_char, mut ptype: *const MOJO_DataType) -> Vec<RawColumnMeta<'a>> {
    let mut columns = Vec::with_capacity(count);
    for _ in 0..count {
        unsafe {
            let cname = CStr::from_ptr(pname.read()).to_string_lossy();
            pname = pname.add(1);
            let ctype = ptype.read();
            ptype = ptype.add(1);
            columns.push(RawColumnMeta { name: cname, column_type: ctype });
        }
    }
    columns
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::daimojo_library::{MOJO_Transform_Flags, MOJO_Transform_Flags_Type, RawPipeline};

    use super::{DaiMojoLibrary, RawModel};

    // const LIBDAIMOJO_SO: &str = "/home/pk/h2o/mojo2/cpp/build/libdaimojo.so";
    const LIBDAIMOJO_SO: &str = "libdaimojo.so";

    #[test]
    fn iris() {
        let lib = DaiMojoLibrary::load(Path::new(LIBDAIMOJO_SO)).unwrap();
        let version = lib.version();
        println!("version: {version}");

        let model = RawModel::load(&lib, "../mojo2/data/iris/pipeline.mojo", ".").unwrap();

        println!("UUID: {}", model.uuid());
        println!("IsValid: {}", model.is_valid());
        println!("TimeCreated: {}", model.time_created_utc());
        let missing_values = &model.missing_values();
        println!("Missing values[{}]: {}", missing_values.len(), missing_values.join(", "));
        let features = model.features();
        println!("Features[{}]:", features.len());
        for column in &features {
            println!("* {} : {:?}", column.name, column.column_type);
        }
        let pipeline = RawPipeline::new(&model, MOJO_Transform_Flags::PREDICT as MOJO_Transform_Flags_Type).unwrap();
        // outputs
        let outputs = pipeline.outputs();
        println!("Outputs[{}]:", outputs.len());
        for column in &outputs {
            println!("* {} : {:?}", column.name, column.column_type);
        }
        //
    }
}
