//! Raw API implementation for interface of shared library "daimojo"
//!
#![allow(non_snake_case)]

use std::borrow::Cow;
use std::ffi::CStr;
use std::io::ErrorKind;
use std::os::raw::c_char;
use std::path::PathBuf;
use chrono::{DateTime, NaiveDateTime, Utc};

use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;

use crate::error;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct MOJO_Model {
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
    flags: u32,
    output_count: usize,
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
    MOJO_UNKNOWN = 1,
    MOJO_FLOAT = 2,
    MOJO_DOUBLE = 3,
    MOJO_INT32 = 4,
    MOJO_INT64 = 5,
    MOJO_STRING = 6,
}

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
    MOJO_NewPipeline: unsafe extern "C" fn(mojo_model: *const MOJO_Model, flags: RawFlags) -> *const MOJO_Pipeline,
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
}

pub struct DaiMojoLibrary {
    api: Container<DaiMojoBindings>,
    version: String,
}

impl DaiMojoLibrary {

    pub fn open(libname: &str) -> crate::error::Result<Self> {
        let lib = PathBuf::from(libname).canonicalize()?;
        let libname = lib.to_str().expect(&format!("Not a valid unicode pathname: {libname}"));
        let version_api: Container<DaiMojoVersionBindings> = unsafe { Container::load(libname) }
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{libname}: {e:?}")))?;
        let version = unsafe { CStr::from_ptr(version_api.mojo_version()) }.to_string_lossy();

        if !version.starts_with("2.") {
            return Err(error::MojoError::UnsupportedApi(libname.to_string(), version.to_string()));
        }
        let container = unsafe { Container::load(libname) };
        log::debug!("Version: {version}");
        let api = container?;
        Ok(Self { api, version: version.to_string()})
    }

    pub fn version(&self) -> Cow<str> {
        Cow::from(&self.version)
    }

    pub fn new_model(&self, filename: &CStr, tf_lib_prefix: &CStr) -> *const MOJO_Model {
        unsafe {
            self.api.MOJO_NewModel(filename.as_ptr(), tf_lib_prefix.as_ptr())
        }
    }
    pub fn delete_model(&self, pipeline: *const MOJO_Model) {
        unsafe { self.api.MOJO_DeleteModel(pipeline) }
    }

    pub fn uuid(&self, pipeline: *const MOJO_Model) -> &CStr {
        unimplemented!()
    }

    pub fn is_valid(&self, pipeline: *const MOJO_Model) -> bool {
        unimplemented!()
    }

    pub fn time_created(&self, pipeline: *const MOJO_Model) -> u64 {
        unimplemented!()
    }

    pub fn missing_values_num(&self, pipeline: *const MOJO_Model) -> usize {
        unimplemented!()
    }

    pub fn missing_values(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unimplemented!()
    }

    pub fn feature_num(&self, pipeline: *const MOJO_Model) -> usize {
        unimplemented!()
    }

    pub fn feature_names(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unimplemented!()
    }

    pub fn feature_types(&self, pipeline: *const MOJO_Model) -> *const MOJO_DataType {
        unimplemented!()
    }

    pub fn output_num(&self, pipeline: *const MOJO_Model) -> usize {
        unimplemented!()
    }

    pub fn output_names(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unimplemented!()
    }

    pub fn output_types(&self, pipeline: *const MOJO_Model) -> *const MOJO_DataType {
        unimplemented!()
    }

    pub fn predict(&self, pipeline: *const MOJO_Model, frame: *const MOJO_Frame, nrow: usize) {
        unimplemented!()
    }

    pub fn new_frame(&self, pipeline: *const MOJO_Pipeline, nrow: usize) -> *const MOJO_Frame {
        unimplemented!()
    }

    pub fn delete_frame(&self, frame: *const MOJO_Frame) {
        unimplemented!()
    }

    pub fn frame_get_row_count(&self, frame: *const MOJO_Frame) -> usize {
        unimplemented!()
    }

    pub fn frame_ncol(&self, frame: *const MOJO_Frame) -> usize {
        unimplemented!()
    }

    pub fn column_buffer(&self, frame: *const MOJO_Frame, colname: *const c_char) -> *mut u8 {
        unimplemented!()
    }
}

type RawFlags = u64;

pub struct RawColumnMeta<'a> {
    pub name: Cow<'a, str>,
    pub column_type: MOJO_DataType,
}

pub struct RawModel<'a> {
    lib: &'a DaiMojoLibrary,
    model_ptr: *const MOJO_Model,
}

impl<'a> RawModel<'a> {
    pub fn load(lib: &'a DaiMojoLibrary, filename: &CStr, tf_lib_prefix: &CStr) -> std::io::Result<Self> {
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
}

impl<'a> RawPipeline<'a> {
    pub fn new(model: &'a RawModel, flags: RawFlags) -> error::Result<Self> {
        let pipeline_ptr = unsafe { model.lib.api.MOJO_NewPipeline(model.model_ptr, flags)};
        Ok(Self {
            lib: model.lib,
            pipeline_ptr,
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
}

impl<'a> Drop for RawPipeline<'a> {
    fn drop(&mut self) {
        unsafe { self.lib.api.MOJO_DeletePipeline(self.pipeline_ptr); }
    }
}

pub struct RawFrame<'a> {
    lib: &'a DaiMojoLibrary,
    frame_ptr: *const MOJO_Frame,
    pipeline_ptr: *const MOJO_Pipeline,
}

impl<'a> RawFrame<'a> {
    pub fn new(pipeline: &'a RawPipeline, nrow: usize) -> error::Result<RawFrame<'a>> {
        let pipeline_ptr = pipeline.pipeline_ptr;
        let frame_ptr = unsafe { pipeline.lib.api.MOJO_Pipeline_NewFrame(pipeline_ptr, nrow) };
        Ok(Self {
            lib: pipeline.lib,
            frame_ptr,
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

    pub unsafe fn output_data(&self, index: usize) -> *const u8 {
        unsafe {
            self.lib.api.MOJO_Output_Data(self.pipeline_ptr, self.frame_ptr, index)
        }
    }
}

impl<'a> Drop for RawFrame<'a> {
    fn drop(&mut self) {
        unsafe { self.lib.api.MOJO_DeleteFrame(self.frame_ptr) };
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

pub trait PArrayOperations<T> {
    fn to_slice<'a>(&self, count: usize) -> &'a [T];
}

impl <T> PArrayOperations<T> for *const T {
    fn to_slice<'a>(&self, count: usize) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(*self, count) }
    }
}

pub trait PCharArrayOperations {
    fn to_vec_cstr<'a>(&self, count: usize) -> Vec<&'a CStr>;
    fn to_vec_string(&self, count: usize) -> Vec<String>;
}

impl PCharArrayOperations for PCharArray {
    fn to_vec_cstr<'a>(&self, count: usize) -> Vec<&'a CStr> {
        let mut vec = Vec::new();
        let slice = self.to_slice(count);
        for &p in slice {
            let s = unsafe { CStr::from_ptr(p) };
            vec.push(s);
        }
        vec
    }

    fn to_vec_string(&self, count: usize) -> Vec<String> {
        let mut vec = Vec::new();
        let slice = self.to_slice(count);
        for &p in slice {
            let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().to_string();
            vec.push(s);
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::path::PathBuf;
    use crate::daimojo_library::{MOJO_Transform_Flags, RawFlags, RawPipeline};

    use super::{DaiMojoLibrary, RawModel};

    // const LIBDAIMOJO_SO: &str = "/home/pk/h2o/mojo2/cpp/build/libdaimojo.so";
    const LIBDAIMOJO_SO: &str = "libdaimojo.so";

    #[test]
    fn iris() {
        let lib = DaiMojoLibrary::open(LIBDAIMOJO_SO).unwrap();
        let version = lib.version();
        println!("version: {version}");

        let filename = PathBuf::from("../mojo2/data/iris/pipeline.mojo");
        let filename = CString::new(filename.to_string_lossy().as_ref()).unwrap();
        let model = RawModel::load(&lib, filename.as_ref(), CString::new("").unwrap().as_ref()).unwrap();

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
        let pipeline = RawPipeline::new(&model, MOJO_Transform_Flags::PREDICT as RawFlags).unwrap();
        // outputs
        let outputs = pipeline.outputs();
        println!("Outputs[{}]:", outputs.len());
        for column in &outputs {
            println!("* {} : {:?}", column.name, column.column_type);
        }
        //
    }
}
