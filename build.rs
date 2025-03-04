use cbindgen;

use std::env;
// use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file("include/simple_web_core.h");
}