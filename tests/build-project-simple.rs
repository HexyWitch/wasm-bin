extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;

use std::path::Path;
use std::fs;

use wasm_build_support::cargo;
use cargo::WasmArtifact;
use wasm_build_support::bindgen;
use wasm_build_support::webpack;

#[test]
fn build_project_simple() {
    let project_dir = Path::new("./test-project-simple");
    std::env::set_current_dir(project_dir).expect("Error setting working directory");

    let mut cargo_options = cargo::BuildOptions::default();
    let artifacts = match cargo::build(&cargo_options) {
        Err(_) => {
            panic!("Errors encountered during cargo build step. Aborting build.");
        }
        Ok(a) => a,
    };
    println!("Finished cargo build step.");

    bindgen::install_if_required(Some(true)).unwrap();
    let mut bins = Vec::new();
    for a in artifacts {
        let (binary, target, path) = match a {
            WasmArtifact::Binary(target, path) => (true, target, path),
            WasmArtifact::Library(target, path) => (false, target, path),
        };
        let (js_out, _) = bindgen::generate(&target, &path).unwrap();
        if binary {
            bins.push((target, js_out));
        }
    }

    webpack::install_if_required(true).unwrap();
    for (target, path) in bins {
        webpack::package_bin(&target, &path).unwrap();
    }

    // fs::remove_dir_all(Path::new("./target/wasm-build"));
    // fs::remove_dir_all(Path::new("./target/wasm32-unknown-unknown"));
}
