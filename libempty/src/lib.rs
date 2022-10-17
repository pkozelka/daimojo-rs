//! Sample and quite trivial implementation of the daimojo api
//!
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::c_char;
use std::os::raw::c_uchar;
use std::ptr;
use std::time::UNIX_EPOCH;

const VERSION: *const u8 = "2.99.99 EMPTY\0".as_ptr();
const UUID: *const u8 = "00000000-0000-0000-0000-000000000000\0".as_ptr();

pub type PCharArray = *const *const c_uchar;
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

#[no_mangle]
extern "C" fn MOJO_Version() -> *const c_uchar {
    VERSION
}

#[allow(non_camel_case_types)]
pub struct MOJO_Model {}

#[allow(non_camel_case_types)]
pub struct MOJO_Frame {}

#[allow(non_camel_case_types)]
pub struct MOJO_Col {}


// added with command
//  sed -n '/^    MOJO_\w\+:/{s:^    ::;s:,$::;s:^\(\w\+\).*fn\((.*$\):#[no_mangle] extern "C"\nfn \1\2 {\n    todo!()\n}\n:;p}' src/daimojo_library.rs >>libempty/src/lib.rs
//  sed -n '/^    MOJO_\w\+:/{s:^    ::;s:,$::;s:^\(\w\+\).*fn\((.*$\):#[no_mangle] extern "C"\nfn \1\2 {\n    println!(" -----> called fn \1()");\n    todo!()\n}\n:;p}' src/daimojo_library.rs >>libempty/src/lib.rs
//  sed -n '/^    MOJO_\w\+:/{s:^    ::;s:,$::;s:^\(\w\+\).*fn(\(.*$\):#[no_mangle] extern "C"\nfn \1(_\2 {\n    println!(" -----> called fn \1()");\n    todo!()\n}\n:;p}' src/daimojo_library.rs >>libempty/src/lib.rs
#[no_mangle] extern "C"
fn MOJO_NewModel(_filename: *const c_char, _tf_lib_prefix: *const c_char) -> *const MOJO_Model {
    println!(" -----> called MOJO_NewModel");
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_DeleteModel(pipeline: *const MOJO_Model) {
    println!(" -----> called fn MOJO_DeleteModel(pipeline=0x{:x})", pipeline as usize);
}

#[no_mangle] extern "C"
fn MOJO_UUID(pipeline: *const MOJO_Model) -> *const c_uchar {
    println!(" -----> called fn MOJO_UUID(pipeline=0x{:x})", pipeline as usize);
    UUID
}

#[no_mangle] extern "C"
fn MOJO_IsValid(pipeline: *const MOJO_Model) -> i32 {
    println!(" -----> called fn MOJO_IsValid(pipeline=0x{:x})", pipeline as usize);
    1
}

#[no_mangle] extern "C"
fn MOJO_TimeCreated(pipeline: *const MOJO_Model) -> u64 {
    println!(" -----> called fn MOJO_TimeCreated(pipeline=0x{:x})", pipeline as usize);
    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

#[no_mangle] extern "C"
fn MOJO_MissingValuesNum(pipeline: *const MOJO_Model) -> usize {
    println!(" -----> called fn MOJO_MissingValuesNum(pipeline=0x{:x})", pipeline as usize);
    0
}

#[no_mangle] extern "C"
fn MOJO_MissingValues(pipeline: *const MOJO_Model) -> PCharArray {
    println!(" -----> called fn MOJO_MissingValues(pipeline=0x{:x})", pipeline as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_FeatureNum(pipeline: *const MOJO_Model) -> usize {
    println!(" -----> called fn MOJO_FeatureNum(pipeline=0x{:x})", pipeline as usize);
    0
}

#[no_mangle] extern "C"
fn MOJO_FeatureNames(pipeline: *const MOJO_Model) -> PCharArray {
    println!(" -----> called fn MOJO_FeatureNames(pipeline=0x{:x})", pipeline as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_FeatureTypes(pipeline: *const MOJO_Model) -> *const MOJO_DataType {
    println!(" -----> called fn MOJO_FeatureTypes(pipeline=0x{:x})", pipeline as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_OutputNum(pipeline: *const MOJO_Model) -> usize {
    println!(" -----> called fn MOJO_OutputNum(pipeline=0x{:x})", pipeline as usize);
    0
}

#[no_mangle] extern "C"
fn MOJO_OutputNames(pipeline: *const MOJO_Model) -> PCharArray {
    println!(" -----> called fn MOJO_OutputNames(pipeline=0x{:x})", pipeline as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_OutputTypes(pipeline: *const MOJO_Model) -> *const MOJO_DataType {
    println!(" -----> called fn MOJO_OutputTypes(pipeline=0x{:x})", pipeline as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_Predict(pipeline: *const MOJO_Model, frame: *const MOJO_Frame) {
    println!(" -----> called fn MOJO_Predict(pipeline=0x{:x}, frame=0x{:x})", pipeline as usize, frame as usize);
}

#[no_mangle] extern "C"
fn MOJO_NewFrame(_cols: *const *const MOJO_Col, _names: PCharArray, count: usize) -> *const MOJO_Frame {
    println!(" -----> called fn MOJO_NewFrame(... count={count})");
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_DeleteFrame(frame: *const MOJO_Frame) {
    println!(" -----> called fn MOJO_DeleteFrame(frame=0x{:x})", frame as usize);
}

#[no_mangle] extern "C"
fn MOJO_FrameNcol(frame: *const MOJO_Frame) -> usize {
    println!(" -----> called fn MOJO_FrameNcol(frame=0x{:x})", frame as usize);
    0
}

#[no_mangle] extern "C"
fn MOJO_GetColByName(frame: *const MOJO_Frame, name: *const c_uchar) -> *const MOJO_Col {
    println!(" -----> called fn MOJO_GetColByName(frame=0x{:x},name='{name:?}')", frame as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_NewCol(datatype: MOJO_DataType, size: usize, data: *mut u8) -> *const MOJO_Col {
    println!(" -----> called fn MOJO_NewCol(type={datatype:?}, size={size}, data=0x{:x})", data as usize);
    ptr::null()
}

#[no_mangle] extern "C"
fn MOJO_DeleteCol(col: *const MOJO_Col) {
    println!(" -----> called fn MOJO_DeleteCol(col=0x{:x})", col as usize);
}

#[no_mangle] extern "C"
fn MOJO_Type(col: *const MOJO_Col) -> MOJO_DataType {
    println!(" -----> called fn MOJO_Type(col=0x{:x})", col as usize);
    MOJO_DataType::MOJO_UNKNOWN
}

#[no_mangle] extern "C"
fn MOJO_Data(col: *const MOJO_Col) -> *mut u8 {
    println!(" -----> called fn MOJO_Data(col=0x{:x})", col as usize);
    ptr::null_mut()
}
