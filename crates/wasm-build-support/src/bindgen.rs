use std::io;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

use util;
use util::prompt_confirm;

const WASM_BINDGEN_GIT_URL: &str = "https://github.com/alexcrichton/wasm-bindgen";
const WASM_BINDGEN_OUT_DIR: &str = "./target/wasm-build/release";
const INSTALL_PROMPT: &str =
    "No installation of wasm-bindgen found. Do you want to install wasm-bindgen? (y/n): ";

#[derive(Debug)]
pub enum Error {
    InstallFailed,
    InstallCommandError(io::Error),
    BindgenFailed,
    BindgenCommandError(io::Error),
    GenerateModuleFailed,
    GenerateModuleCommandError(io::Error),
    PromptError(util::Error),
    CreateTargetDirectoryError(io::Error),
}

pub fn install_if_required(skip_prompt: Option<bool>) -> Result<(), Error> {
    // check if wasm-bindgen CLI tool is installed, if not, ask the user to install it
    match Command::new("wasm-bindgen")
        .arg("-h")
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                let skip_prompt = skip_prompt.unwrap_or(false);
                let do_install =
                    skip_prompt || prompt_confirm(INSTALL_PROMPT).map_err(Error::PromptError)?;
                if do_install {
                    install()?;
                    Ok(())
                } else {
                    Err(Error::BindgenCommandError(e))
                }
            }
            _ => Err(Error::BindgenCommandError(e)),
        },
    }
}

fn install() -> Result<(), Error> {
    let mut install = Command::new("cargo")
        .arg("install")
        .arg("--git")
        .arg(WASM_BINDGEN_GIT_URL)
        .spawn()
        .map_err(Error::InstallCommandError)?;

    match install.wait() {
        Ok(status) => match status.success() {
            true => Ok(()),
            false => Err(Error::InstallFailed),
        },
        Err(e) => Err(Error::InstallCommandError(e)),
    }
}

pub fn generate_wasm(target_name: &str, input_file: &Path) -> Result<PathBuf, Error> {
    // Create target directory if it doesn't exist
    let mut out_dir = PathBuf::from(WASM_BINDGEN_OUT_DIR);
    out_dir.push(target_name);
    match fs::read_dir(&out_dir) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                fs::create_dir_all(&out_dir).map_err(Error::CreateTargetDirectoryError)?;
            }
            _ => return Err(Error::BindgenCommandError(e)),
        },
    }

    let mut bindgen = Command::new("wasm-bindgen")
        .arg(&input_file)
        .arg("--out-dir")
        .arg(&out_dir)
        .spawn()
        .map_err(Error::BindgenCommandError)?;

    match bindgen.wait() {
        Ok(status) => match status.success() {
            true => {}
            false => return Err(Error::BindgenFailed),
        },
        Err(e) => return Err(Error::BindgenCommandError(e)),
    }

    let mut out_file = out_dir;
    out_file.push(format!("{}_wasm.wasm", target_name));
    Ok(out_file)
}
