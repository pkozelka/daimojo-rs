//! Raw API implementation for interface of shared library "daimojo"
//!
#![allow(non_snake_case)]

use std::alloc::Layout;
use std::os::raw::c_char;
use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;
use std::io::ErrorKind;
use std::ffi::CStr;
use std::path::PathBuf;

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
pub struct DaiMojoVersionBindings {
    #[dlopen2_name = "MOJO_Version"]
    mojo_version: unsafe extern "C" fn() -> *const c_char,
}

#[derive(dlopen2_derive::WrapperApi)]
pub struct DaiMojoBindings {
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
    MOJO_Pipeline_Predict: unsafe extern "C" fn(pipeline: *const MOJO_Model, frame: *const MOJO_Frame, nrow: usize),
    // Frame
    MOJO_Pipeline_NewFrame: unsafe extern "C" fn(pipeline: *const MOJO_Model, nrow: usize) -> *const MOJO_Frame,
    MOJO_DeleteFrame: unsafe extern "C" fn(frame: *const MOJO_Frame),
    MOJO_FrameNcol: unsafe extern "C" fn(frame: *const MOJO_Frame) -> usize,
    MOJO_GetColByName: unsafe extern "C" fn(frame: *const MOJO_Frame, name: *const c_char) -> *const MOJO_Col,
    // Column
    MOJO_NewCol: unsafe extern "C" fn(datatype: MOJO_DataType, size: usize, data: *mut u8) -> *const MOJO_Col,
    MOJO_DeleteCol: unsafe extern "C" fn(col: *const MOJO_Col),
    MOJO_Type: unsafe extern "C" fn(col: *const MOJO_Col) -> MOJO_DataType,
    MOJO_Data: unsafe extern "C" fn(col: *const MOJO_Col) -> *mut u8,
    // DEPRECATED APIS
    MOJO_Predict: unsafe extern "C" fn(pipeline: *const MOJO_Model, frame: *const MOJO_Frame),
    MOJO_NewFrame: unsafe extern "C" fn(cols: *const *const MOJO_Col, names: PCharArray, count: usize) -> *const MOJO_Frame,
}

pub struct DaiMojoLibrary {
    api: Container<DaiMojoBindings>,
    version: &'static CStr
}

impl DaiMojoLibrary {

    pub fn open(libname: &str) -> Result<Self, std::io::Error> {
        let lib = PathBuf::from(libname).canonicalize()?;
        let libname = lib.to_str().expect(&format!("Not a valid unicode pathname: {libname}"));
        let version_api: Container<DaiMojoVersionBindings> = unsafe { Container::load(libname) }
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{libname}: {e:?}")))?;
        let version = unsafe { CStr::from_ptr(version_api.mojo_version()) };

        if version.to_bytes()[0] != b'2' {
            return Err(std::io::Error::new(ErrorKind::InvalidInput, format!("{libname}: Not a supported API inside version '{}'", version.to_string_lossy())))
        }
        let container = unsafe { Container::load(libname) };
        let api = container
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{libname}: {e:?}")))?;
        Ok(Self { api, version})
    }

    pub fn version(&self) -> &CStr {
        self.version
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

    pub fn predict(&self, pipeline: *const MOJO_Model, frame: *const MOJO_Frame, nrow: usize) {
        unsafe { self.api.MOJO_Pipeline_Predict(pipeline, frame, nrow) }
    }

    pub fn new_frame(&self, pipeline: *const MOJO_Model, nrow: usize) -> *const MOJO_Frame {
        unsafe { self.api.MOJO_Pipeline_NewFrame(pipeline, nrow) }
    }

    pub fn delete_frame(&self, frame: *const MOJO_Frame) {
        unsafe { self.api.MOJO_DeleteFrame(frame)}
    }

    pub fn get_col_by_name(&self, frame: *const MOJO_Frame, name: *const c_char) -> *const MOJO_Col {
        unsafe { self.api.MOJO_GetColByName(frame, name)}
    }

    pub fn frame_ncol(&self, frame: *const MOJO_Frame) -> usize {
        unsafe { self.api.MOJO_FrameNcol(frame) }
    }

    pub fn new_col(&self, datatype: MOJO_DataType, size: usize) -> *const MOJO_Col {
        //TODO: Bad! the implementation should itself allocate the memory, because Rust has its own allocator
        // and it's not integrated with the one from the library
        let data = Self::alloc_data(&datatype, size);
        unsafe { self.api.MOJO_NewCol(datatype, size, data) }
    }

    fn alloc_data(datatype: &MOJO_DataType, size: usize) -> *mut u8 {
        let layout = match datatype {
            MOJO_DataType::MOJO_FLOAT => Layout::array::<f32>(size),
            MOJO_DataType::MOJO_DOUBLE => Layout::array::<f64>(size),
            MOJO_DataType::MOJO_INT32 => Layout::array::<i32>(size),
            MOJO_DataType::MOJO_INT64 => Layout::array::<i64>(size),
            MOJO_DataType::MOJO_STRING => Layout::array::<*const c_char>(size),
            _ => panic!("Unsupported type")
        };
        let layout = layout.expect("Invalid memory layout");
        unsafe { std::alloc::alloc_zeroed(layout) }
    }

    pub fn _delete_col(&self, col: *const MOJO_Col) {
        unsafe { self.api.MOJO_DeleteCol(col) }
    }

    pub fn data(&self, col: *const MOJO_Col) -> *mut u8 {
        unsafe { self.api.MOJO_Data(col) }
    }

    pub fn _datatype(&self, col: *const MOJO_Col) -> MOJO_DataType {
        unsafe { self.api.MOJO_Type(col) }
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

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::path::PathBuf;
    use super::{DaiMojoLibrary, PArrayOperations, PCharArrayOperations};

    // const LIBDAIMOJO_SO: &str = "lib/linux_x64/libdaimojo.so";
    const LIBDAIMOJO_SO: &str = "libdaimojo.so";

    #[test]
    fn iris() {
        let lib = DaiMojoLibrary::open(LIBDAIMOJO_SO).unwrap();
        let version = lib.version().to_string_lossy();
        println!("version: {version}");

        let filename = PathBuf::from("../mojo2/data/iris/pipeline.mojo");
        let filename = CString::new(filename.to_string_lossy().as_ref()).unwrap();
        let pipeline = lib.new_model(filename.as_ref(), CString::new("").unwrap().as_ref());
        println!("UUID: {}", lib.uuid(pipeline).to_string_lossy());
        println!("IsValid: {}", lib.is_valid(pipeline));
        println!("TimeCreated: {}", lib.time_created(pipeline));
        let missing_values = lib.missing_values(pipeline).to_vec_string(lib.missing_values_num(pipeline));
        println!("Missing values>: {}", missing_values.join(", "));
        let icnt = lib.feature_num(pipeline);
        println!("Features[{icnt}]:");
        let names = lib.feature_names(pipeline).to_vec_string(icnt);
        let types = lib.feature_types(pipeline).to_slice(icnt);
        for i in 0..icnt {
            println!("* {} : {:?}", &names[i], types[i]);
        }
        let ocnt = lib.output_num(pipeline);
        println!("Outputs[{ocnt}]:");
        let names = lib.output_names(pipeline).to_vec_string(ocnt);
        let types = lib.output_types(pipeline).to_slice(ocnt);
        for i in 0..ocnt {
            println!("* {} : {:?}", &names[i], types[i]);
        }
        //
        lib.delete_model(pipeline);
        println!("deleted");
    }
}
