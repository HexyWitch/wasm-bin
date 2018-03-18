extern crate clap;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;

mod run;

use clap::{App, Arg, ArgMatches, SubCommand};

use wasm_build_support::build;

fn shared_args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("bin")
            .long("bin")
            .value_name("NAME")
            .help("Build only the specified binary")
            .takes_value(true),
        Arg::with_name("features")
            .long("features")
            .value_name("FEATURES")
            .help("Space-separated list of features to also build")
            .takes_value(true),
    ]
}

fn build_options(matches: &ArgMatches) -> build::Options {
    let mut build_options = build::Options::default();
    if let Some(bin) = matches.value_of("bin") {
        build_options.bin = Some(bin.to_string());
    }
    if let Some(features) = matches.value_of("features") {
        build_options.features = Some(features.to_string());
    }
    build_options
}

fn main() {
    let app = App::new("wasm-build")
        .subcommand(SubCommand::with_name("build").args(&shared_args()))
        .subcommand(SubCommand::with_name("run").args(&shared_args()))
        .get_matches();

    if let Some(matches) = app.subcommand_matches("build") {
        let options = build_options(matches);
        build::build(&options).unwrap();
    } else if let Some(matches) = app.subcommand_matches("run") {
        let options = build_options(matches);
        let targets = build::build(&options).unwrap();
        for target in targets {
            match target.ty {
                build::PackageType::Binary => {
                    run::serve(target.name, target.path);
                }
                _ => {}
            }
        }
    }
}
