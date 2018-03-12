use std::path::PathBuf;

use cargo;
use bindgen;
use webpack;
use cargo::WasmArtifact;

#[derive(Debug)]
pub enum Error {
    CargoBuildError(cargo::Error),
    BindgenError(bindgen::Error),
    WebpackError(webpack::Error),
}

#[derive(Default)]
pub struct Options {
    pub bin: Option<String>,
    pub features: Option<String>,
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

pub fn build(options: &Options) -> Result<Vec<TargetPackage>, Error> {
    println!("wasm-build: Starting cargo build step");
    let cargo_options = cargo::BuildOptions {
        bin: options.bin.clone(),
        features: options.features.clone(),
    };
    let artifacts = cargo::build(&cargo_options).map_err(Error::CargoBuildError)?;

    bindgen::install_if_required(Some(true)).map_err(Error::BindgenError)?;
    let mut bins = Vec::new();
    let mut libs = Vec::new();
    for a in artifacts {
        let (binary, target, path) = match a {
            WasmArtifact::Binary(target, path) => (true, target, path),
            WasmArtifact::Library(target, path) => (false, target, path),
        };

        println!("wasm-build: Generate js bindings for target '{}'", target);
        let (mut js_out, _) = bindgen::generate(&target, &path).map_err(Error::BindgenError)?;
        if binary {
            js_out.pop();
            bins.push((target, js_out));
        } else {
            libs.push((target, js_out));
        }
    }

    webpack::install_if_required(true).unwrap();

    let mut targets = Vec::new();
    for (target, path) in bins {
        println!("wasm-build: Package binary target '{}'", target);
        let dir = webpack::package_bin(&target, &path).map_err(Error::WebpackError)?;
        targets.push(TargetPackage {
            ty: PackageType::Binary,
            name: target,
            path: dir,
        });
    }
    for (target, js_path) in libs {
        println!("wasm-build: Package library target '{}'", target);
        let dir = webpack::package(&target, &js_path).map_err(Error::WebpackError)?;
        targets.push(TargetPackage {
            ty: PackageType::Library,
            name: target,
            path: dir,
        });
    }
    Ok(targets)
}
