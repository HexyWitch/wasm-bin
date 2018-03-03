use std::io;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

const WASM_BINDGEN_GIT_URL: &str = "https://github.com/alexcrichton/wasm-bindgen";
const WASM_BINDGEN_OUT_DIR: &str = "target/wasm-build/release/.";

#[derive(Debug)]
pub struct Error;

fn prompt_confirm(text: &str) -> bool {
    println!("{}", text);

    let read_in = || {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        match buf.trim_right() {
            "y" | "Y" => Some(true),
            "n" | "N" => Some(false),
            _ => None,
        }
    };
    loop {
        match read_in() {
            Some(v) => return v,
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
                let do_install = skip_prompt || prompt_confirm("No installation of wasm-bindgen found. Do you want to install wasm-bindgen? (y/n): ");
                if do_install {
                    install()?;
                    Ok(())
                } else {
                    Err(Error)
                }
            }
            _ => Err(Error),
        },
    }
}

fn install() -> Result<(), Error> {
    let mut install = Command::new("cargo")
        .arg("install")
        .arg("--git")
        .arg(WASM_BINDGEN_GIT_URL)
        .spawn()
        .unwrap();

    match install.wait() {
        Ok(_) => Ok(()),
        Err(_) => Err(Error),
    }
}

pub fn generate_wasm(input_file: &Path) -> Result<PathBuf, Error> {
    // Create target directory if it doesn't exist
    let out_dir = PathBuf::from(WASM_BINDGEN_OUT_DIR);
    match fs::read_dir(&out_dir) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                fs::create_dir_all(&out_dir).unwrap();
            }
            _ => return Err(Error),
        },
    }

    let mut bindgen = Command::new("wasm-bindgen")
        .arg(&input_file)
        .arg("--out-dir")
        .arg(WASM_BINDGEN_OUT_DIR)
        .spawn()
        .unwrap();

    match bindgen.wait() {
        Ok(_) => {}
        Err(_) => return Err(Error),
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
        .unwrap();

    match wasm2es6js.wait() {
        Ok(_) => {}
        Err(_) => return Err(Error),
    }

    Ok(out_file)
}
