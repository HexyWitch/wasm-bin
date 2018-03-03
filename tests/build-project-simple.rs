extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;

use std::path::Path;

use wasm_build_support::cargo;
use cargo::WasmArtifact;
use wasm_build_support::wasm_bindgen;

#[test]
fn build_project_simple() {
    let project_dir = Path::new("./test-project-simple");
    std::env::set_current_dir(project_dir).expect("Error setting working directory");

    let mut cargo_options = cargo::BuildOptions::default();
    cargo_options.lib = true;
    let artifacts = match cargo::build(&cargo_options) {
        Err(e) => {
            panic!(
                "Errors encountered during cargo build step. Aborting build. {:?}",
                e
            );
        }
        Ok(a) => a,
    };

    wasm_bindgen::install_if_required().unwrap();
    for a in artifacts {
        let path = match a {
            WasmArtifact::Binary(_) => {
                panic!("Found binary in crate build-project simple");
            }
            WasmArtifact::Library(path) => path,
        };
        let generated_wasm = wasm_bindgen::generate_wasm(&path).unwrap();
        wasm_bindgen::generate_js_module(&generated_wasm).unwrap();
    }

    std::fs::remove_dir_all(Path::new("target")).unwrap();
}
