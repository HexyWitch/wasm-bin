use std::io;
use std::path::PathBuf;

use bindgen;
use cargo;
use cargo::WasmArtifact;

#[cfg(test)]
const SKIP_PROMPT: bool = true;
#[cfg(not(test))]
const SKIP_PROMPT: bool = false;

#[derive(Debug)]
pub enum Error {
    CargoBuildError(cargo::Error),
    BindgenError(bindgen::Error),
    ExportMainError(io::Error),
}

#[derive(Default)]
pub struct Options {
    pub package: Option<String>,
    pub all: bool,
    pub exclude: Option<String>,
    pub jobs: Option<String>,
    pub lib: bool,
    pub bin: Option<String>,
    pub bins: bool,
    pub example: Option<String>,
    pub all_targets: bool,
    pub release: bool,
    pub features: Option<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub manifest_path: Option<String>,
    pub verbose: bool,
    pub quiet: bool,
    pub frozen: bool,
    pub locked: bool,
    pub cargo_flags: Option<String>,
}

#[derive(Debug)]
pub enum PackageType {
    Binary,
    Library,
}

#[derive(Debug)]
pub struct TargetPackage {
    pub ty: PackageType,
    pub name: String,
    pub path: PathBuf,
}

pub fn build(options: &Options) -> Result<Vec<TargetPackage>, Error> {
    println!("wasm-bin: Starting cargo build step");
    let cargo_options = cargo::BuildOptions {
        package: options.package.clone(),
        all: options.all.clone(),
        exclude: options.exclude.clone(),
        jobs: options.jobs.clone(),
        lib: options.lib.clone(),
        bin: options.bin.clone(),
        bins: options.bins.clone(),
        example: options.example.clone(),
        all_targets: options.all_targets.clone(),
        release: options.release.clone(),
        features: options.features.clone(),
        all_features: options.all_features.clone(),
        no_default_features: options.no_default_features.clone(),
        manifest_path: options.manifest_path.clone(),
        verbose: options.verbose.clone(),
        quiet: options.quiet.clone(),
        frozen: options.frozen.clone(),
        locked: options.locked.clone(),
        cargo_flags: options.cargo_flags.clone(),
    };
    let artifacts = cargo::build(&cargo_options).map_err(Error::CargoBuildError)?;

    bindgen::install_if_required(Some(SKIP_PROMPT)).map_err(Error::BindgenError)?;
    let mut targets = Vec::new();
    for a in artifacts {
        let (package_type, target, path) = match a {
            WasmArtifact::Binary(target, path) => (PackageType::Binary, target, path),
            WasmArtifact::Library(target, path) => (PackageType::Library, target, path),
        };

        println!("wasm-bin: Generate js bindings for target '{}'", target);
        let (mut js_out, _) = bindgen::generate(&target, &path).map_err(Error::BindgenError)?;
        targets.push(TargetPackage {
            ty: package_type,
            name: target,
            path: js_out,
        });
    }

    Ok(targets)
}
