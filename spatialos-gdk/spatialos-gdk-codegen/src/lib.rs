#![recursion_limit = "256"]

extern crate walkdir;

extern crate syn;
#[macro_use]
extern crate quote;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

mod command;
mod component;
mod event;
mod field;
mod global;
mod json;
mod schema_type;
mod user_type;

use global::Global;
use quote::Tokens;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub fn codegen<P: AsRef<Path>, O: AsRef<Path> + Clone>(json_dir: P, output_file: O) {
    if generate_json_ast(&json_dir) {
        let json_files: Vec<PathBuf> = WalkDir::new(json_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("json"))
            .map(|e| PathBuf::from(e.path()))
            .collect();

        let tokens = parse_json(json_files);

        ::std::fs::write(output_file.clone(), tokens.to_string()).expect("Unable to write file");

        format_file(output_file);
    }
}

fn format_file<P: AsRef<Path>>(path: P) {
    let output = Command::new("rustfmt")
        .arg(path.as_ref().to_str().unwrap())
        .output()
        .expect("ls command failed to start");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!("Formatting generated code failed: {} {}", stdout, stderr);
    }
}

fn generate_json_ast<P: AsRef<Path>>(json_dir: &P) -> bool {
    let output = Command::new("spatial")
        .args(&[
            "process_schema",
            "generate",
            "--output",
            json_dir.as_ref().to_str().unwrap(),
            "--language",
            "ast_json",
            "--force",
        ])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);
    println!("{}", stderr);

    if !output.status.success() {
        panic!(
            "spatial process_schema generate failed: {} {}",
            stdout, stderr
        );
    } else {
        true
    }
}

pub fn parse_json<P: AsRef<Path>>(paths: Vec<P>) -> Tokens {
    let mut collection = json::JsonCollection::default();
    for path in paths.into_iter() {
        let result = json::parse_json(path).unwrap();
        collection.append(result);
    }

    let schema = Global::from(collection);
    schema.get_code()
}

fn to_rust_qualified_name(qualified_name: &str) -> String {
    let parts: Vec<&str> = qualified_name.split(".").collect();
    format!("::schema::{}", parts.join("::"))
}
