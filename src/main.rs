extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod cargo;
mod wasm_bindgen;

use clap::{App, Arg};

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

    let artifacts = match cargo::build(&matches) {
        Err(_) => {
            println!("Errors encountered during cargo build step. Aborting build.");
            return;
        }
        Ok(a) => a,
    };
    println!("Finished cargo build step.");

    wasm_bindgen::install_if_required().unwrap();
    for a in artifacts {
        println!(
            "Generate wasm-bindgen bindings for artifact: {}",
            a.clone().into_os_string().to_str().unwrap()
        );
        let generated_wasm = wasm_bindgen::generate_wasm(&a).unwrap();

        println!("Bundle wasm into a js module");
        wasm_bindgen::generate_js_module(&generated_wasm).unwrap();
    }
}
