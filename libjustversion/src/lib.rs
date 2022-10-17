//! A dynamic library exposing only function `MOJO_Version`
//!
//! This is useful to demonstrate that the client code can load this single symbol and based on it,
//! can make decisions about loading further sets of symbols (APIs)
//!
//! Resources:
//!
//! `eh_personality`
//! * https://www.reddit.com/r/rust/comments/9dk284/what_is_rust_eh_personality_and_can_we_get_rid_of/
//! * https://stackoverflow.com/questions/16597350/what-is-an-exception-handling-personality-function
//! * https://blog.knoldus.com/os-in-rust-an-executable-that-runs-on-bare-metal-part-1/
//! * https://os.phil-opp.com/freestanding-rust-binary/#the-eh-personality-language-item
//! * https://docs.rs/rrt0/0.1.3/rrt0/fn.rust_eh_personality.html

// Further minimization - this is possible only in nightly:
// #![no_std]
// #![feature(lang_items)]
// #[lang = "eh_personality"] #[no_mangle] fn rust_eh_personality() {}
// #[panic_handler] fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! { loop {} }

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn MOJO_Version() -> *const u8 {
    "JUST VERSION\0".as_ptr()
}
