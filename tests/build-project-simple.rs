extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;

use std::path::Path;
use std::fs;
use std::io::Write;

use wasm_build_support::build;

static CARGO_TOML: &str = r#"
[package]
name = "test_project_simple"
version = "0.0.1"

[dependencies]
wasm-bindgen = { git = 'https://github.com/alexcrichton/wasm-bindgen' }
"#;

#[test]
fn build_project_simple() {
    let project_dir = Path::new("./tests/test-project-simple");
    std::env::set_current_dir(project_dir).expect("Error setting working directory");

    let options = build::Options::default();
    let manifest_path = Path::new("Cargo.toml");
    {
        let mut manifest = fs::File::create(manifest_path).unwrap();
        manifest.write_all(CARGO_TOML.as_bytes()).unwrap();
    }

    build::build(&options).unwrap();
    
    fs::remove_file(manifest_path).unwrap()
}
