use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use dlopen2::wrapper::Container;
use dlopen2::wrapper::WrapperApi;


struct MojoPipeline {}

#[derive(dlopen2_derive::WrapperApi)]
struct RawApi {
    #[dlopen2_name = "MOJO_Version"]
    version: unsafe extern "C" fn() -> *const c_char,
    #[dlopen2_name = "MOJO_NewModel"]
    new_model: unsafe extern "C" fn(filename: *const c_char, tf_lib_prefix: *const c_char) -> *const MojoPipeline,
    #[dlopen2_name = "MOJO_FeatureNum"]
    feature_num: unsafe extern "C" fn(pipeline: *const MojoPipeline) -> isize,
    #[dlopen2_name = "MOJO_FeatureNames"]
    feature_names: unsafe extern "C" fn(pipeline: *const MojoPipeline) -> *const *const c_char,
    #[dlopen2_name = "MOJO_OutputNum"]
    output_num: unsafe extern "C" fn(pipeline: *const MojoPipeline) -> isize,
    #[dlopen2_name = "MOJO_OutputNames"]
    output_names: unsafe extern "C" fn(pipeline: *const MojoPipeline) -> *const *const c_char,
    #[dlopen2_name = "MOJO_DeleteModel"]
    delete_model: unsafe extern "C" fn(pipeline: *const MojoPipeline),
}

fn main() {
    // using bindgen: https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a
    // https://github.com/shepmaster/rust-ffi-omnibus + http://jakegoulding.com/rust-ffi-omnibus/
    // char ** : http://adam.younglogic.com/2019/03/accessing-c-arrays-of-string-from-rust/


    // println!("DAIMOJO Version: {}", mojo_version().to_string_lossy());

    let libname = "/home/pk/h2o/jna-simple-mojo2/lib/linux_x64/libdaimojo.so";
    // let libname = "target/libdemo.so";
    // let lib = dlopen::raw::Library::open(libname).unwrap();
    // let symbol = unsafe { lib.symbol("MOJO_Version") }.unwrap();
    let cont: Container<RawApi> =
        unsafe {
            Container::load(libname) }
            .expect("Could not open library or load symbols");

    // print version
    let v = unsafe { cont.version() };
    let v = unsafe {CStr::from_ptr(v)};
    let v = v.to_string_lossy();
    println!("Version: {}", v);

    // load pipeline
    let filename = PathBuf::from("../mojo2/data/iris/pipeline.mojo");
    let filename = CString::new(filename.to_string_lossy().as_ref()).unwrap();
    let tf_lib_prefix = CString::new("").unwrap();
    let pipeline = unsafe { cont.new_model(filename.into_raw(), tf_lib_prefix.into_raw()) }; //TODO somehow give it type (MojoPipeline)
    println!("ok");

    let icnt = unsafe { cont.feature_num(pipeline) };
    println!("feature num: {icnt}");
    let ocnt = unsafe { cont.output_num(pipeline) };
    println!("output num: {ocnt}");

    unsafe {
        let mpa = cont.feature_names(pipeline);
        if mpa != ptr::null() {
            for i in 0..icnt {
                let p: *const c_char = *(mpa.offset(i));
                let name = CStr::from_ptr(p);
                println!("{i}. {}", name.to_string_lossy());
            }
        }
    };

    let outputs = charpp_to_strs(unsafe { cont.output_names(pipeline) }, ocnt);
    println!("Outputs:");
    for name in outputs {
        println!("* {name}");
    }

    unsafe { cont.delete_model(pipeline) }
    println!("deleted");
}


fn charpp_to_strs<'a>(charpp: *const *const c_char, count: isize) -> Vec<Cow<'a,str>> {
    let mut vec = Vec::new();
    unsafe {
        if charpp != ptr::null() {
            for i in 0..count {
                let p: *const c_char = *(charpp.offset(i as isize));
                let name = CStr::from_ptr(p);
                vec.push(name.to_string_lossy());
            }
        }
    }
    vec
}
