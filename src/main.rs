extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[allow(unused)]
mod cargo;

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

    let artifacts = cargo::build(&matches).unwrap();
    println!("Finished cargo build step. Found binary artifacts:");
    for a in artifacts {
        println!("{}", a.into_os_string().to_str().unwrap());
    }
}
