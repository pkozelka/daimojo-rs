fn main() {
    println!("cargo:rustc-link-lib=daimojo");
    println!("cargo:rustc-link-search=lib/linux_x64");
}
