extern crate clap;
extern crate serde;
extern crate serde_json;

extern crate wasm_build_support;

use clap::{App, Arg};

use wasm_build_support::cargo;
use cargo::WasmArtifact;
use wasm_build_support::bindgen;

fn main() {
    let matches = App::new("wasm-build")
        .arg(
            Arg::with_name("bin")
                .long("bin")
                .value_name("NAME")
                .help("Build only the specified binary")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .value_name("FEATURES")
                .help("Space-separated list of features to also build")
                .takes_value(true),
        )
        .get_matches();

    let mut cargo_options = cargo::BuildOptions::default();
    if let Some(bin) = matches.value_of("bin") {
        cargo_options.bin = Some(bin.to_string());
    }
    if let Some(features) = matches.value_of("features") {
        cargo_options.features = Some(features.to_string());
    }
    let artifacts = match cargo::build(&cargo_options) {
        Err(_) => {
            panic!("Errors encountered during cargo build step. Aborting build.");
        }
        Ok(a) => a,
    };
    println!("Finished cargo build step.");

    bindgen::install_if_required(None).unwrap();
    for a in artifacts {
        let (binary, path) = match a {
            WasmArtifact::Binary(path) => (true, path),
            WasmArtifact::Library(path) => (false, path),
        };
        println!(
            "Generate wasm-bindgen bindings for artifact: {}",
            path.clone().into_os_string().to_str().unwrap()
        );
        let generated_wasm = bindgen::generate_wasm(&path).unwrap();

        println!("Bundle wasm into a js module");
        bindgen::generate_js_module(&generated_wasm).unwrap();

        if binary {
            // Produce a bundled html file
        } else {
            // Produce an es6 js module
        }
    }
}
