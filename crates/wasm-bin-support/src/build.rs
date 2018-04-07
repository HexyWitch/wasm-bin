use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use bindgen;
use cargo;
use cargo::WasmArtifact;
use webpack;

#[cfg(test)]
const SKIP_PROMPT: bool = true;
#[cfg(not(test))]
const SKIP_PROMPT: bool = false;

#[derive(Debug)]
pub enum Error {
    CargoBuildError(cargo::Error),
    BindgenError(bindgen::Error),
    ExportMainError(io::Error),
    WebpackError(webpack::Error),
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

pub enum PackageType {
    Binary,
    Library,
}

pub struct TargetPackage {
    pub ty: PackageType,
    pub name: String,
    pub path: PathBuf,
}

fn export_main(js: &Path) -> Result<(), Error> {
    let mut js_file = OpenOptions::new()
        .append(true)
        .open(js)
        .map_err(Error::ExportMainError)?;
    js_file
        .write_all(
            "export function main() {
            wasm.main();
        }"
                .as_bytes(),
        )
        .map_err(Error::ExportMainError)?;
    Ok(())
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
    let mut bins = Vec::new();
    let mut libs = Vec::new();
    for a in artifacts {
        let (binary, target, path) = match a {
            WasmArtifact::Binary(target, path) => (true, target, path),
            WasmArtifact::Library(target, path) => (false, target, path),
        };

        println!("wasm-bin: Generate js bindings for target '{}'", target);
        let (mut js_out, _) = bindgen::generate(&target, &path).map_err(Error::BindgenError)?;
        if binary {
            export_main(&js_out)?;
            js_out.pop();
            bins.push((target, js_out));
        } else {
            libs.push((target, js_out));
        }
    }

    webpack::install_if_required(SKIP_PROMPT).unwrap();

    let mut targets = Vec::new();
    for (target, path) in bins {
        println!("wasm-bin: Package binary target '{}'", target);
        let dir = webpack::package_bin(&target, &path).map_err(Error::WebpackError)?;
        targets.push(TargetPackage {
            ty: PackageType::Binary,
            name: target,
            path: dir,
        });
    }
    for (target, js_path) in libs {
        println!("wasm-bin: Package library target '{}'", target);
        let dir = webpack::package(&target, &js_path).map_err(Error::WebpackError)?;
        targets.push(TargetPackage {
            ty: PackageType::Library,
            name: target,
            path: dir,
        });
    }
    Ok(targets)
}