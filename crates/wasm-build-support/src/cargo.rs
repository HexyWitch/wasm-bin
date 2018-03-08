use std::process::{Command, Stdio};
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use serde_json;
use serde::de;

const BIN_TARGET_KIND_ID: &str = "bin";
const LIB_TARGET_KIND_ID: &str = "cdylib";

#[derive(Debug)]
pub enum Error {
    RunCommandError(io::Error),
    CompileErrors,
    DeserializeOutputError,
    CaptureStdoutError,
    StdoutLineError(io::Error),
    SerializeMessageError(serde_json::error::Error),
    UnexpectedFileCountError,
}

pub struct BuildOptions {
    pub bin: Option<String>,
    pub features: Option<String>,
    pub manifest_path: Option<String>,
}

impl Default for BuildOptions {
    fn default() -> BuildOptions {
        BuildOptions {
            bin: None,
            features: None,
            manifest_path: None,
        }
    }
}

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
#[derive(Clone, Deserialize, Serialize)]
struct DiagnosticCode {
    code: String,
    explanation: Option<String>,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Serialize)]
struct DiagnosticSpanLine {
    text: String,
    highlight_start: usize,
    highlight_end: usize,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Serialize)]
struct DiagnosticSpanMacroExpansion {
    span: DiagnosticSpan,
    macro_decl_name: String,
    def_site_span: Option<DiagnosticSpan>,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Serialize)]
struct DiagnosticSpan {
    file_name: String,
    byte_start: u32,
    byte_end: u32,
    line_start: usize,
    line_end: usize,
    column_start: usize,
    column_end: usize,
    is_primary: bool,
    text: Vec<DiagnosticSpanLine>,
    label: Option<String>,
    suggested_replacement: Option<String>,
    expansion: Option<Box<DiagnosticSpanMacroExpansion>>,
}

#[allow(unused)]
#[derive(Clone, Deserialize, Serialize)]
struct Diagnostic {
    message: String,
    code: Option<DiagnosticCode>,
    /// "error: internal compiler error", "error", "warning", "note", "help".
    level: String,
    spans: Vec<DiagnosticSpan>,
    children: Vec<Diagnostic>,
    rendered: Option<String>,
}

#[allow(unused)]
#[derive(Deserialize)]
struct CargoFromCompiler {
    package_id: PackageId,
    target: Target,
    message: Diagnostic,
}

#[allow(unused)]
#[derive(Deserialize)]
struct BuildScript {
    pub package_id: PackageId,
    pub linked_libs: Vec<String>,
    pub linked_paths: Vec<String>,
    pub cfgs: Vec<String>,
    pub env: Vec<(String, String)>,
}

enum CargoBuildOutput {
    FromCompiler(CargoFromCompiler),
    Artifact(Artifact),
    BuildScript(BuildScript),
}

fn parse_cargo_output(line: &str) -> Result<CargoBuildOutput, (Error)> {
    if let Ok(from_compiler) = serde_json::from_str::<CargoFromCompiler>(line) {
        return Ok(CargoBuildOutput::FromCompiler(from_compiler));
    }
    if let Ok(artifact) = serde_json::from_str::<Artifact>(line) {
        return Ok(CargoBuildOutput::Artifact(artifact));
    }
    if let Ok(script_result) = serde_json::from_str::<BuildScript>(line) {
        return Ok(CargoBuildOutput::BuildScript(script_result));
    }

    Err(Error::DeserializeOutputError)
}

#[derive(Clone)]
pub enum WasmArtifact {
    Binary(String, PathBuf),
    Library(String, PathBuf),
}

// Returns a list of paths to binary wasm artifacts produced by the cargo build command
pub fn build(options: &BuildOptions) -> Result<Vec<WasmArtifact>, Error> {
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped())
        .arg("build")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--release")
        .args(&["--message-format", "json"]);

    if let Some(ref bin) = options.bin {
        cmd.arg("--bin").arg(bin);
    }
    if let Some(ref features) = options.features {
        cmd.arg("--features").arg(features);
    }
    if let Some(ref manifest_path) = options.manifest_path {
        cmd.arg("--manifest-path").arg(manifest_path);
    }

    let child = cmd.spawn().map_err(|e| Error::RunCommandError(e))?;

    let stdout = BufReader::new(child.stdout.ok_or_else(|| Error::CaptureStdoutError)?);

    let mut artifacts = Vec::new();
    let mut errors = Vec::new();
    let bin_id = String::from(BIN_TARGET_KIND_ID);
    let lib_id = String::from(LIB_TARGET_KIND_ID);
    for line in stdout.lines() {
        let line = line.map_err(|e| Error::StdoutLineError(e))?;
        let output = match parse_cargo_output(&line) {
            Ok(o) => o,
            Err(e) => {
                println!("Could not parse output:\n{}", &line);
                return Err(e);
            }
        };
        match output {
            CargoBuildOutput::FromCompiler(from_compiler) => {
                match from_compiler.message.level.as_str() {
                    "error" => {
                        let error = from_compiler.message.clone().rendered.map_or(
                            serde_json::to_string(&from_compiler.message)
                                .map_err(Error::SerializeMessageError)?,
                            |v| v.to_string(),
                        );
                        println!("{}", error);
                        errors.push(error);
                    }
                    _ => {}
                }
            }
            CargoBuildOutput::Artifact(mut artifact) => {
                if artifact.filenames.len() != 1 {
                    return Err(Error::UnexpectedFileCountError);
                }
                if artifact.target.kind.contains(&bin_id) {
                    artifacts.push(WasmArtifact::Binary(
                        artifact.target.name,
                        artifact.filenames.pop().unwrap(),
                    ));
                } else if artifact.target.kind.contains(&lib_id) {
                    artifacts.push(WasmArtifact::Library(
                        artifact.target.name,
                        artifact.filenames.pop().unwrap(),
                    ));
                }
            }
            CargoBuildOutput::BuildScript(_) => {}
        }
    }

    match errors.len() {
        0 => Ok(artifacts),
        _ => Err(Error::CompileErrors),
    }
}
