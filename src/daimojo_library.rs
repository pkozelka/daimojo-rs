//! Raw API implementation for interface of shared library "daimojo"
//!
#![allow(non_snake_case)]

use std::alloc::Layout;
use std::os::raw::c_char;
use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;
use std::io::ErrorKind;
use std::ffi::CStr;

#[allow(non_camel_case_types)]
pub struct MOJO_Model {}

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

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoBindings {
    #[dlopen2_name = "MOJO_Version"]
    mojo_version: unsafe extern "C" fn() -> *const c_char,
    // Model
    MOJO_NewModel: unsafe extern "C" fn(filename: *const c_char, tf_lib_prefix: *const c_char) -> *const MOJO_Model,
    MOJO_DeleteModel: unsafe extern "C" fn(pipeline: *const MOJO_Model),
    MOJO_UUID: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const c_char,
    MOJO_IsValid: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> i32,
    MOJO_TimeCreated: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> u64,
    MOJO_MissingValuesNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_MissingValues: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_FeatureNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_FeatureNames: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_FeatureTypes: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const MOJO_DataType,
    MOJO_OutputNum: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> usize,
    MOJO_OutputNames: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> PCharArray,
    MOJO_OutputTypes: unsafe extern "C" fn(pipeline: *const MOJO_Model) -> *const MOJO_DataType,
    MOJO_Predict: unsafe extern "C" fn(pipeline: *const MOJO_Model, frame: *const MOJO_Frame),
    // Frame
    MOJO_NewFrame: unsafe extern "C" fn(cols: *const *const MOJO_Col, names: PCharArray, count: usize) -> *const MOJO_Frame,
    MOJO_DeleteFrame: unsafe extern "C" fn(frame: *const MOJO_Frame),
    MOJO_FrameNcol: unsafe extern "C" fn(frame: *const MOJO_Frame) -> usize,
    MOJO_GetColByName: unsafe extern "C" fn(frame: *const MOJO_Frame, name: *const c_char) -> *const MOJO_Col,
    // Column
    MOJO_NewCol: unsafe extern "C" fn(datatype: MOJO_DataType, size: usize, data: *mut u8) -> *const MOJO_Col,
    MOJO_DeleteCol: unsafe extern "C" fn(col: *const MOJO_Col),
    MOJO_Type: unsafe extern "C" fn(col: *const MOJO_Col) -> MOJO_DataType,
    MOJO_Data: unsafe extern "C" fn(col: *const MOJO_Col) -> *mut u8,
}

pub struct DaiMojoLibrary {
    api: Container<DaiMojoBindings>,
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

    pub fn feature_types(&self, pipeline: *const MOJO_Model) -> *const MOJO_DataType {
        unsafe { self.api.MOJO_FeatureTypes(pipeline) }
    }

    pub fn output_num(&self, pipeline: *const MOJO_Model) -> usize {
        unsafe { self.api.MOJO_OutputNum(pipeline) }
    }

    pub fn output_names(&self, pipeline: *const MOJO_Model) -> PCharArray {
        unsafe { self.api.MOJO_OutputNames(pipeline) }
    }

    pub fn output_types(&self, pipeline: *const MOJO_Model) -> *const MOJO_DataType {
        unsafe { self.api.MOJO_OutputTypes(pipeline) }
    }

    pub fn predict(&self, pipeline: *const MOJO_Model, frame: *const MOJO_Frame) {
        unsafe { self.api.MOJO_Predict(pipeline, frame) }
    }

    pub fn new_frame(&self, cols: *const *const MOJO_Col, names: PCharArray, count: usize) -> *const MOJO_Frame {
        unsafe { self.api.MOJO_NewFrame(cols, names, count) }
    }

    pub fn get_col_by_name(&self, frame: *const MOJO_Frame, name: *const c_char) -> *const MOJO_Col {
        unsafe { self.api.MOJO_GetColByName(frame, name)}
    }

    pub fn new_col(&self, datatype: MOJO_DataType, size: usize) -> *const MOJO_Col {
        //TODO: Bad! the implementation should itself allocate the memory, because Rust has its own allocator
        // and it's not integrated with the one from the library
        let data = Self::alloc_data(&datatype, size);
        unsafe { self.api.MOJO_NewCol(datatype, size, data) }
    }

    pub fn alloc_data(datatype: &MOJO_DataType, size: usize) -> *mut u8 {
        let layout = match datatype {
            MOJO_DataType::MOJO_FLOAT => Layout::array::<f32>(size),
            MOJO_DataType::MOJO_DOUBLE => Layout::array::<f64>(size),
            MOJO_DataType::MOJO_INT32 => Layout::array::<i32>(size),
            MOJO_DataType::MOJO_INT64 => Layout::array::<i64>(size),
            MOJO_DataType::MOJO_STRING => Layout::array::<*const c_char>(size),
            _ => panic!("Unsupported type")
        };
        let layout = layout.expect("Invalid memory layout");
        let data = unsafe { std::alloc::alloc_zeroed(layout) };
        data
    }

    pub fn data(&self, col: *const MOJO_Col) -> *mut u8 {
        unsafe { self.api.MOJO_Data(col) }
    }
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
