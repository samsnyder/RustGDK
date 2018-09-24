use std::env;
use std::path::PathBuf;

extern crate spatialos_gdk_codegen;

fn main() {
    let json_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("json");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs");
    spatialos_gdk_codegen::codegen(json_path, out_path);
}
