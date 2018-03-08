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

pub fn build(options: &Options) -> Result<(), Error> {
    println!("wasm-build: Starting cargo build step");
    let cargo_options = cargo::BuildOptions {
        bin: options.bin.clone(),
        features: options.features.clone(),
    };
    let artifacts = cargo::build(&cargo_options).map_err(Error::CargoBuildError)?;

    bindgen::install_if_required(Some(true)).map_err(Error::BindgenError)?;
    let mut bins = Vec::new();
    for a in artifacts {
        let (binary, target, path) = match a {
            WasmArtifact::Binary(target, path) => (true, target, path),
            WasmArtifact::Library(target, path) => (false, target, path),
        };
        println!("wasm-build: Generate js bindings for target '{}'", target);
        let (js_out, _) = bindgen::generate(&target, &path).map_err(Error::BindgenError)?;
        if binary {
            bins.push((target, js_out));
        }
    }

    webpack::install_if_required(true).unwrap();
    for (target, path) in bins {
        println!("wasm-build: Package binary target '{}'", target);
        webpack::package_bin(&target, &path).unwrap();
    }
    Ok(())
}