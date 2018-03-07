extern crate clap;
extern crate serde;
extern crate serde_json;

extern crate wasm_build_support;

use clap::{App, Arg};

use wasm_build_support::cargo;
use cargo::WasmArtifact;
use wasm_build_support::bindgen;
use wasm_build_support::webpack;

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
}
