use std::ffi::CString;
use std::path::PathBuf;
use crate::daimojo::DaiMojoLibrary;

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
    println!("Features:");
    for name in lib.feature_names(pipeline, lib.feature_num(pipeline)) {
        println!("* {}", name.to_string_lossy());
    }
    println!("Outputs:");
    for name in lib.output_names(pipeline, lib.output_num(pipeline)) {
        println!("* {}", name.to_string_lossy());
    }
    lib.delete_model(pipeline);
    println!("deleted");
}

mod daimojo;
