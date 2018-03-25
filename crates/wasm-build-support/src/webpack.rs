use std::io;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::process::Command;
use std::path::{Path, PathBuf};

use util;
use util::prompt_confirm;

const YARN_MISSING: &str =
    "No installation of yarn found. yarn is required to install webpack. https://yarnpkg.com/";
const WEBPACK_INSTALL_PROMPT: &str =
    "No installation of webpack found. Do you want to install webpack? (y/n)";

#[cfg(target_os = "windows")]
const YARN_CMD: &str = "yarn.cmd";
#[cfg(not(target_os = "windows"))]
const YARN_CMD: &str = "yarn";

#[cfg(target_os = "windows")]
const WEBPACK_CMD: &str = "webpack.cmd";
#[cfg(not(target_os = "windows"))]
const WEBPACK_CMD: &str = "webpack";

// To run batch files on windows they must be run through cmd
// https://github.com/rust-lang/rust/issues/42791
fn yarn_command() -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/k").arg("yarn.cmd");
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    Command::new("yarn")
}
fn webpack_command() -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/k").arg("webpack.cmd");
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    Command::new("webpack")
}

#[derive(Debug)]
pub enum Error {
    YarnMissing,
    YarnCommandError(io::Error),
    PromptError(util::Error),
    WebpackMissing,
    WebpackCommandError(io::Error),
    InstallFailed,
    InstallCommandError(io::Error),
    PackageFailed,
    PackageCommandError(io::Error),
    RemoveExistingDistError(io::Error),
    CopyJsModulesError(io::Error),
    CopyHtmlIndexError(io::Error),
    WriteJsIndexError(io::Error),
    WriteHtmlIndexError(io::Error),
}

pub fn install_if_required(skip_prompt: bool) -> Result<(), Error> {
    // This will not actually correctly run yarn on windows because running batch
    // files as processes is not correctly supported. It will however still return
    // the NotFound error if the command is not found, which is the only thing
    // we want out of running this command.
    // This is really hacky, figure out a better way of checking for installed batch scripts.
    let yarn = Command::new(YARN_CMD).arg("-v").output();
    match yarn {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                panic!(YARN_MISSING);
            }
            _ => return Err(Error::YarnCommandError(e)),
        },
        _ => {}
    }

    // Check if webpack is installed, and if not, prompt the user to install it
    // This will not correctly run `webpack -v`, but will produce the NotFound error
    // if webpack is not found.
    match Command::new(WEBPACK_CMD).arg("-v").output() {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                if skip_prompt
                    || prompt_confirm(WEBPACK_INSTALL_PROMPT).map_err(Error::PromptError)?
                {
                    install().unwrap();
                    Ok(())
                } else {
                    Err(Error::WebpackCommandError(e))
                }
            }
            _ => Err(Error::WebpackCommandError(e)),
        },
        _ => Ok(()),
    }
}

fn install() -> Result<(), Error> {
    println!("wasm-build: Installing webpack");
    match yarn_command()
        .arg("global")
        .arg("add")
        .arg("webpack")
        .arg("webpack-cli")
        .output()
    {
        Ok(output) => match output.status.success() {
            true => Ok(()),
            false => {
                println!("{}", String::from_utf8_lossy(&output.stderr));
                Err(Error::InstallFailed)
            }
        },
        Err(e) => Err(Error::InstallCommandError(e)),
    }
}

fn path_exists(path: &Path) -> bool {
    match fs::metadata(path) {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => false,
            _ => true,
        },
        _ => true,
    }
}

fn copy_js_modules(target_name: &str, out_dir: &Path) -> Result<(), Error> {
    let js_dir = Path::new("./js");
    if !path_exists(js_dir) {
        return Ok(());
    }

    println!("Find js modules");
    for dir_entry in fs::read_dir(Path::new("./js")).map_err(Error::CopyJsModulesError)? {
        let path = dir_entry.unwrap().path();
        if path.as_os_str().to_str().unwrap().ends_with(".js") {
            fs::copy(&path, out_dir.join(path.file_name().unwrap()))
                .map_err(Error::CopyJsModulesError)?;
        }
    }

    let target_js_dir = js_dir.join(target_name);
    if !path_exists(&target_js_dir) {
        return Ok(());
    }
    for dir_entry in fs::read_dir(target_js_dir).map_err(Error::CopyJsModulesError)? {
        let path = dir_entry.unwrap().path();
        if path.ends_with(".js") {
            fs::copy(path, out_dir).map_err(Error::CopyJsModulesError)?;
        }
    }

    Ok(())
}

fn copy_js_index(target_name: &str, out_dir: &Path) -> Result<Option<PathBuf>, Error> {
    let js_dir = Path::new("./js").join(target_name).join("index.js");
    if !path_exists(&js_dir) {
        return Ok(None);
    }

    let out_file = out_dir.join(js_dir.file_name().unwrap());
    fs::copy(&js_dir, &out_file).map_err(Error::CopyHtmlIndexError)?;
    Ok(Some(out_file))
}

fn create_js_index(target_name: &str, dir: &Path) -> Result<PathBuf, Error> {
    let content = format!(
        r#"
        void async function () {{
            const js = await import("./{}");
            js.main()
        }}();
        "#,
        target_name
    );
    let js_path: PathBuf = [dir, Path::new("index.js")].iter().collect();
    let mut js_index = File::create(&js_path).map_err(Error::WriteHtmlIndexError)?;
    js_index
        .write_all(content.as_bytes())
        .map_err(Error::WriteHtmlIndexError)?;
    js_index.flush().map_err(Error::WriteHtmlIndexError)?;

    Ok(js_path)
}

pub fn package(target_name: &str, entry: &Path) -> Result<PathBuf, Error> {
    let mut out_dir = entry.with_file_name("");
    out_dir.push("dist");

    // Remove dist folder if it already exists
    match fs::remove_dir_all(&out_dir) {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {}
            _ => return Err(Error::RemoveExistingDistError(e)),
        },
        _ => {}
    }

    copy_js_modules(target_name, &entry.with_file_name(""))?;

    let out_file: PathBuf = [&out_dir, Path::new(&format!("{}.js", target_name))]
        .iter()
        .collect();
    // Package the js index file into a bundle
    match webpack_command()
        .arg(entry)
        .arg("--output")
        .arg(&out_file)
        .arg("--mode")
        .arg("development")
        .output()
    {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                return Err(Error::PackageFailed);
            }
        }
        Err(e) => return Err(Error::PackageCommandError(e)),
    }

    Ok(out_file)
}

pub fn package_bin(target_name: &str, dir: &Path) -> Result<PathBuf, Error> {
    let js_index = match copy_js_index(target_name, dir)? {
        Some(f) => f,
        None => create_js_index(target_name, dir)?,
    };

    let out_file = package(target_name, &js_index)?;
    Ok(out_file)
}
