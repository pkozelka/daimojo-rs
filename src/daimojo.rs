//! Raw API implementation for interface of shared library "daimojo"
//!
use std::os::raw::c_char;
use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;
use std::io::ErrorKind;
use std::ffi::CStr;

#[repr(C)]
pub struct MOJO_Model {}

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoLibraryRawApi {
    #[dlopen2_name = "MOJO_Version"]
    mojo_version: unsafe extern "C" fn() -> *const c_char,
    #[dlopen2_name = "MOJO_NewModel"]
    mojo_new_model: unsafe extern "C" fn(filename: *const c_char, tf_lib_prefix: *const c_char) -> *const MOJO_Model,
    #[dlopen2_name = "MOJO_FeatureNum"]
    mojo_feature_num: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    #[dlopen2_name = "MOJO_FeatureNames"]
    mojo_feature_names: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const *const c_char,
    #[dlopen2_name = "MOJO_OutputNum"]
    mojo_output_num: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    #[dlopen2_name = "MOJO_OutputNames"]
    mojo_output_names: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const *const c_char,
    #[dlopen2_name = "MOJO_DeleteModel"]
    mojo_delete_model: unsafe extern "C" fn(pipeline: *const MOJO_Model),
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
            self.api.mojo_new_model(filename.as_ptr(), tf_lib_prefix.as_ptr())
        }
    }
    pub fn delete_model(&self, pipeline: *const MOJO_Model) {
        unsafe { self.api.mojo_delete_model(pipeline) }
    }

    pub fn feature_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.mojo_feature_num(pipeline) }
    }

    pub fn feature_names(&self, pipeline: *const MOJO_Model, count: usize) -> Vec<&CStr> {
        unsafe { self.api.mojo_feature_names(pipeline) }.to_vec(count)
    }

    pub fn output_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.mojo_output_num(pipeline) }
    }

    pub fn output_names(&self, pipeline: *const MOJO_Model, count: usize) -> Vec<&CStr> {
        unsafe { self.api.mojo_output_names(pipeline) }.to_vec(count)
    }
}

trait PCharArray {
    fn to_vec<'a>(&self, count: usize) -> Vec<&'a CStr>;
}

impl PCharArray for *const *const c_char {
    fn to_vec<'a>(&self, count: usize) -> Vec<&'a CStr> {
        charpp_to_vec(*self, count)
    }
}

pub fn charpp_to_vec<'a>(charpp: *const *const c_char, count: usize) -> Vec<&'a CStr> {
    let mut vec = Vec::new();
    let slice = unsafe { std::slice::from_raw_parts( charpp, count) };
    for &p in slice {
        let s = unsafe { CStr::from_ptr(p) };
        vec.push(s);
    }
    vec
}
