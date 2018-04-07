extern crate clap;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate wasm_bin_support;

mod run;

use clap::{App, Arg, ArgMatches, SubCommand};

use wasm_bin_support::build;

fn shared_args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("package")
            .long("package")
            .value_name("SPEC")
            .help("Package to build")
            .takes_value(true),
        Arg::with_name("exclude")
            .value_name("SPEC")
            .help("Exclude packages from the build")
            .takes_value(true),
        Arg::with_name("j")
            .short("j")
            .long("jobs")
            .value_name("N")
            .help("Number of parallel jobs, defaults to # of CPUs")
            .takes_value(true),
        Arg::with_name("bin")
            .long("bin")
            .value_name("NAME")
            .help("Build only the specified binary")
            .takes_value(true),
        Arg::with_name("release")
            .long("release")
            .help("Build artifacts in release mode, with optimizations"),
        Arg::with_name("features")
            .long("features")
            .value_name("FEATURES")
            .help("Space-separated list of features to also build")
            .takes_value(true),
        Arg::with_name("all-features")
            .long("all-features")
            .help("Build all available features"),
        Arg::with_name("no-default-features")
            .long("no-default-features")
            .help("Do not build the `default` feature"),
        Arg::with_name("manifest-path")
            .long("manifest-path")
            .value_name("PATH")
            .help("Path to the manifest to compile")
            .takes_value(true),
        Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Use verbose output (-vv very verbose/build.rs output)"),
        Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .help("No output printed to stdout"),
        Arg::with_name("frozen")
            .long("frozen")
            .help("Require Cargo.lock and cache are up to date"),
        Arg::with_name("locked")
            .long("locked")
            .help("Require Cargo.lock is up to date"),
        Arg::with_name("cargo-flags")
            .short("Z")
            .value_name("FLAG")
            .help("Unstable (nightly-only) flags to Cargo")
            .takes_value(true),
    ]
}

fn build_args<'a, 'b>() -> Vec<Arg<'a, 'b>> {
    vec![
        Arg::with_name("all")
            .long("all")
            .help("Build all packages in the workspace"),
        Arg::with_name("lib")
            .long("lib")
            .help("Build only this package's library"),
        Arg::with_name("bins")
            .long("bins")
            .help("Build all binaries"),
        Arg::with_name("all-targets")
            .long("all-targets")
            .help("Build all targets (lib and bin targets by default)"),
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
    build::Options {
        package: matches.value_of("package").map(String::from),
        all: matches.is_present("all"),
        exclude: matches.value_of("exclude").map(String::from),
        jobs: matches.value_of("jobs").map(String::from),
        lib: matches.is_present("lib"),
        bin: matches.value_of("bin").map(String::from),
        bins: matches.is_present("bins"),
        all_targets: matches.is_present("all-targets"),
        release: matches.is_present("release"),
        features: matches.value_of("features").map(String::from),
        all_features: matches.is_present("all-features"),
        no_default_features: matches.is_present("no-default-features"),
        manifest_path: matches.value_of("manifest-path").map(String::from),
        verbose: matches.is_present("verbose"),
        quiet: matches.is_present("quiet"),
        frozen: matches.is_present("frozen"),
        locked: matches.is_present("locked"),
        cargo_flags: matches.value_of("Z").map(String::from),
    }
}

fn main() {
    let app = App::new("wasm-bin")
        .subcommand(
            SubCommand::with_name("build")
                .args(&shared_args())
                .args(&build_args()),
        )
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
