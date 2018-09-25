extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();

    let worker_package_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("worker_sdk");

    let package_name = if target.contains("windows") {
        "c-static-x86_64-msvc_md-win32"
    } else if target.contains("apple") {
        "c-static-x86_64-clang_libcpp-macos"
    } else if target.contains("linux") {
        "c-static-x86_64-gcc_libstdcpp-linux"
    } else {
        panic!("Unknown platform {}", target);
    };

    unpack_worker_package("worker_sdk", package_name, worker_package_dir.clone());

    println!(
        "cargo:rustc-link-search={}",
        worker_package_dir.join("lib").display()
    );

    let libs = if target.contains("windows") {
       vec![
            "worker",
            "grpc++",
            "grpc",
            "gpr",
            "libprotobuf",
            "RakNetLibStatic",
            "ssl",
            "zlibstatic",
        ]
    } else {
        vec![
            "worker",
            "grpc++",
            "grpc",
            "gpr",
            "protobuf",
            "RakNetLibStatic",
            "ssl",
            "z",
        ]
    };

    for lib in libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    let bindings = bindgen::Builder::default()
        .constified_enum_module(".*")
        .disable_untagged_union()
        .header(
            worker_package_dir
                .join("include/improbable/c_worker.h")
                .to_str()
                .unwrap(),
        )
        .header(
            worker_package_dir
                .join("include/improbable/c_schema.h")
                .to_str()
                .unwrap(),
        )
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn unpack_worker_package<P: AsRef<Path>>(package_type: &str, package_name: &str, directory: P) {
    let output = Command::new("spatial")
        .current_dir(env::var("OUT_DIR").unwrap())
        .args(&[
            "worker_package",
            "unpack-to",
            package_type,
            package_name,
            directory.as_ref().to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);
    println!("{}", stderr);

    if !output.status.success() {
        panic!(
            "spatial worker_package unpack-to failed: {} {}",
            stdout, stderr
        );
    }
}
