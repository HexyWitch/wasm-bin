use std::io;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

const WASM_BINDGEN_GIT_URL: &str = "https://github.com/alexcrichton/wasm-bindgen";
const WASM_BINDGEN_OUT_DIR: &str = "./target/wasm-build/release";

#[derive(Debug)]
pub enum Error {
    InstallFailed,
    InstallCommandError(io::Error),
    BindgenFailed,
    BindgenCommandError(io::Error),
    GenerateModuleFailed,
    GenerateModuleCommandError(io::Error),
    ReadLineError(io::Error),
    CreateTargetDirectoryError(io::Error),
}

fn prompt_confirm(text: &str) -> Result<bool, Error> {
    println!("{}", text);

    let read_in = || {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .map_err(Error::ReadLineError)?;
        Ok(match buf.trim_right() {
            "y" | "Y" => Some(true),
            "n" | "N" => Some(false),
            _ => None,
        })
    };
    loop {
        match read_in()? {
            Some(v) => return Ok(v),
            _ => {}
        }
    }
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
                let do_install = skip_prompt || prompt_confirm("No installation of wasm-bindgen found. Do you want to install wasm-bindgen? (y/n): ")?;
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

pub fn generate_wasm(input_file: &Path) -> Result<PathBuf, Error> {
    // Create target directory if it doesn't exist
    let out_dir = PathBuf::from(WASM_BINDGEN_OUT_DIR);
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
        .arg(WASM_BINDGEN_OUT_DIR)
        .spawn()
        .map_err(Error::BindgenCommandError)?;

    match bindgen.wait() {
        Ok(status) => match status.success() {
            true => {}
            false => return Err(Error::BindgenFailed),
        },
        Err(e) => return Err(Error::BindgenCommandError(e)),
    }

    let mut out_file = out_dir.clone();
    out_file.push(input_file.file_name().unwrap());
    out_file.set_extension("");
    let file_name = out_file.file_name().unwrap().to_str().unwrap().to_string();
    out_file.set_file_name(format!("{}_wasm.wasm", file_name));
    println!("{:?}", out_file);
    Ok(out_file)
}

pub fn generate_js_module(input_file: &Path) -> Result<PathBuf, Error> {
    let out_file = {
        let mut path = input_file.to_path_buf();
        path.set_extension("js");
        path
    };
    let mut wasm2es6js = Command::new("wasm2es6js")
        .arg(input_file)
        .arg("-o")
        .arg(&out_file)
        .arg("--base64")
        .spawn()
        .map_err(Error::GenerateModuleCommandError)?;

    match wasm2es6js.wait() {
        Ok(status) => match status.success() {
            true => Ok(out_file),
            false => Err(Error::GenerateModuleFailed),
        },
        Err(e) => Err(Error::GenerateModuleCommandError(e)),
    }
}
