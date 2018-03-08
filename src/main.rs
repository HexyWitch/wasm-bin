extern crate clap;
extern crate serde;
extern crate serde_json;

extern crate wasm_build_support;

use clap::{App, Arg};

use wasm_build_support::build;

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

    let mut build_options = build::Options::default();
    if let Some(bin) = matches.value_of("bin") {
        build_options.bin = Some(bin.to_string());
    }
    if let Some(features) = matches.value_of("features") {
        build_options.features = Some(features.to_string());
    }

    build::build(&build_options).unwrap();
}
