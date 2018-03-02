use std::io;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

const WASM_BINDGEN_GIT_URL: &str = "https://github.com/alexcrichton/wasm-bindgen";
const WASM_BINDGEN_OUT_DIR: &str = "target/wasm-build/release/";

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

pub fn install_if_required() -> Result<(), Error> {
    // check if wasm-bindgen CLI tool is installed, if not, ask the user to install it
    match Command::new("wasm-bindgen")
        .arg("-h")
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                let do_install = prompt_confirm("No installation of wasm-bindgen found. Do you want to install wasm-bindgen? (y/n): ");
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

pub fn process_file(file: PathBuf) -> Result<(), Error> {
    // Create target directory if it doesn't exist
    let path = Path::new(WASM_BINDGEN_OUT_DIR);
    match fs::read_dir(path) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                fs::create_dir_all(path).unwrap();
            }
            _ => return Err(Error),
        },
    }

    let mut bindgen = Command::new("wasm-bindgen")
        .arg(file.into_os_string().to_str().unwrap())
        .arg("--out-dir")
        .arg(WASM_BINDGEN_OUT_DIR)
        .spawn()
        .unwrap();

    match bindgen.wait() {
        Ok(_) => Ok(()),
        Err(_) => Err(Error),
    }
}
