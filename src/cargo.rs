use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use clap::ArgMatches;
use serde_json;
use serde::de;

const BIN_TARGET_KIND_ID: &str = "bin";

#[derive(Debug)]
pub struct Error;

#[derive(Debug)]
struct PackageId {
    name: String,
    version: String,
    source_id: String,
}

impl<'de> de::Deserialize<'de> for PackageId {
    fn deserialize<D>(d: D) -> Result<PackageId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        let mut s = string.splitn(3, ' ');
        Ok(PackageId {
            name: s.next().unwrap().to_string(),
            version: s.next()
                .ok_or_else(|| de::Error::custom("invalid PackageId"))?
                .to_string(),
            source_id: s.next()
                .ok_or_else(|| de::Error::custom("invalid PackageId"))?
                .to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct Target {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Profile {
    opt_level: String,
    debuginfo: Option<u32>,
    debug_assertions: bool,
    overflow_checks: bool,
    test: bool,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    package_id: PackageId,
    target: Target,
    profile: Profile,
    features: Vec<String>,
    filenames: Vec<PathBuf>,
    fresh: bool,
}

#[allow(unused)]
#[derive(Deserialize)]
struct CargoFromCompiler {
    package_id: PackageId,
    target: Target,
    message: serde_json::Value,
}

enum CargoBuildOutput {
    FromCompiler(CargoFromCompiler),
    Artifact(Artifact),
}

fn parse_cargo_output(line: &str) -> Result<CargoBuildOutput, Error> {
    if let Ok(from_compiler) = serde_json::from_str::<CargoFromCompiler>(line) {
        return Ok(CargoBuildOutput::FromCompiler(from_compiler));
    }
    if let Ok(artifact) = serde_json::from_str::<Artifact>(line) {
        return Ok(CargoBuildOutput::Artifact(artifact));
    }

    Err(Error)
}

// Returns a list of paths to binary wasm artifacts produced by the cargo build command
pub fn build(matches: &ArgMatches) -> Result<Vec<PathBuf>, Error> {
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped())
        .arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--release")
        .args(&["--message-format", "json"]);

    if let Some(bin) = matches.value_of("bin") {
        cmd.arg("--bin").arg(bin);
    }

    if let Some(features) = matches.value_of("features") {
        cmd.arg("--features").arg(features);
    }

    let child = cmd.spawn().map_err(|_| Error)?;

    let stdout = BufReader::new(child.stdout.unwrap());

    let mut bins = Vec::new();
    let bin_id = String::from(BIN_TARGET_KIND_ID);
    for line in stdout.lines() {
        match parse_cargo_output(&line.unwrap())? {
            CargoBuildOutput::FromCompiler(from_compiler) => {
                // TODO: Figure out how to pretty print this
                println!("{}", serde_json::to_string(&from_compiler.message).unwrap());
            }
            CargoBuildOutput::Artifact(mut artifact) => {
                if artifact.target.kind.contains(&bin_id) {
                    bins.append(&mut artifact.filenames);
                }
            }
        }
    }

    Ok(bins)
}
