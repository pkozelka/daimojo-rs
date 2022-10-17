//! A dynamic library exposing only function `MOJO_Version`
//!
//! This is useful to demonstrate that the client code can load this single symbol and based on it,
//! can make decisions about loading further sets of symbols (APIs)

use std::os::raw::c_uchar;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn MOJO_Version() -> *const c_uchar {
    "JUST VERSION\0".as_ptr()
}
