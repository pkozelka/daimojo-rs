//! Raw API implementation for interface of shared library "daimojo"
//!
#![allow(non_snake_case)]

use std::os::raw::c_char;
use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;
use std::io::ErrorKind;
use std::ffi::CStr;

#[allow(non_camel_case_types)]
pub struct MOJO_Model {}

/// An alias for array of C strings (`char **`)
pub type PCharArray = *const *const c_char;

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoLibraryRawApi {
    #[dlopen2_name = "MOJO_Version"]
    mojo_version: unsafe extern "C" fn() -> *const c_char,
    // Model
    MOJO_NewModel: unsafe extern "C" fn(filename: *const c_char, tf_lib_prefix: *const c_char) -> *const MOJO_Model,
    MOJO_UUID: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const c_char,
    MOJO_IsValid: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> i32,
    MOJO_TimeCreated: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> u64,
    MOJO_MissingValuesNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_MissingValues: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_FeatureNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_FeatureNames: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_OutputNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_OutputNames: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_DeleteModel: unsafe extern "C" fn(pipeline: *const MOJO_Model),
    // Frame
    // Column
}

pub struct DaiMojoLibrary {
    api: Container<DaiMojoLibraryRawApi>,
}

impl DaiMojoLibrary {

    pub fn open(libname: &str) -> Result<Self, std::io::Error> {
        let container = unsafe { Container::load(libname) };
        let api = container
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{libname}: {e:?}")))?;
        Ok(Self { api })
    }

    pub fn version(&self) -> &CStr {
        let v = unsafe { self.api.mojo_version() };
        unsafe {CStr::from_ptr(v)}
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
        unsafe { CStr::from_ptr(self.api.MOJO_UUID(pipeline)) }
    }

    pub fn is_valid(&self, pipeline: *const MOJO_Model) -> i32 {
        unsafe { self.api.MOJO_IsValid(pipeline) }
    }

    pub fn time_created(&self, pipeline: *const MOJO_Model) -> u64 {
        unsafe { self.api.MOJO_TimeCreated(pipeline) }
    }

    pub fn missing_values_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.MOJO_MissingValuesNum(pipeline) }
    }

    pub fn missing_values(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unsafe { self.api.MOJO_MissingValues(pipeline) }
    }

    pub fn feature_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.MOJO_FeatureNum(pipeline) }
    }

    pub fn feature_names(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unsafe { self.api.MOJO_FeatureNames(pipeline) }
    }

    pub fn output_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.MOJO_OutputNum(pipeline) }
    }

    pub fn output_names(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unsafe { self.api.MOJO_OutputNames(pipeline) }
    }
}

pub trait PCharArrayOperations {
    fn to_slice<'a>(&self, count: usize) -> &'a [*const c_char];
    fn to_vec_cstr<'a>(&self, count: usize) -> Vec<&'a CStr>;
    fn to_vec_string(&self, count: usize) -> Vec<String>;
}

impl PCharArrayOperations for PCharArray {
    fn to_slice<'a>(&self, count: usize) -> &'a [*const c_char] {
        unsafe { std::slice::from_raw_parts( *self, count) }
    }

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
