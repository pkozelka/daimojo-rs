use std::ffi::CString;
use std::path::PathBuf;
use crate::daimojo_library::{DaiMojoLibrary, PArrayOperations, PCharArrayOperations};

fn main() {
    // using bindgen: https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a
    // https://github.com/shepmaster/rust-ffi-omnibus + http://jakegoulding.com/rust-ffi-omnibus/
    // char ** : http://adam.younglogic.com/2019/03/accessing-c-arrays-of-string-from-rust/
    //      https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html
    //      https://stackoverflow.com/questions/48657360/change-c-array-element-via-rust-ffi


    let lib = DaiMojoLibrary::open("lib/linux_x64/libdaimojo.so").unwrap();
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

mod daimojo_library;
mod daimojo;
