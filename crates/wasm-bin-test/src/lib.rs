extern crate wasm_bin_support;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use wasm_bin_support::build;
use wasm_bin_support::build::Options as BuildOptions;

fn path_exists(path: &Path) -> bool {
    match fs::metadata(path) {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => false,
            _ => true,
        },
        _ => true,
    }
}

#[derive(Debug)]
pub enum Error {
    SetWorkingDirError(io::Error),
    CreateDirError(io::Error),
    CreateFileError(String, io::Error),
    WriteFileError(String, io::Error),
    BuildError(build::Error),
}

pub struct TestProject {
    files: HashMap<String, String>,
    build_options: build::Options,
}

impl TestProject {
    pub fn new(build_options: BuildOptions) -> TestProject {
        TestProject {
            files: HashMap::new(),
            build_options,
        }
    }

    pub fn file(&mut self, path: &str, file_contents: &str) -> &mut Self {
        self.files
            .insert(path.to_string(), file_contents.to_string());
        self
    }

    pub fn build(&mut self, out_dir: &Path) -> Result<(), Error> {
        if !path_exists(out_dir) {
            fs::create_dir_all(out_dir).map_err(Error::CreateDirError)?
        }
        std::env::set_current_dir(out_dir).map_err(Error::SetWorkingDirError)?;
        self.write()?;

        build::build(&self.build_options).map_err(Error::BuildError)?;

        Ok(())
    }

    fn write(&mut self) -> Result<(), Error> {
        for (path_name, file_contents) in self.files.iter() {
            let path = Path::new(&path_name);
            if !path_exists(path.parent().unwrap()) {
                fs::create_dir_all(path.parent().unwrap()).map_err(Error::CreateDirError)?;
            }

            let mut file =
                File::create(path).map_err(|e| Error::CreateFileError(path_name.clone(), e))?;
            file.write_all(file_contents.as_bytes())
                .map_err(|e| Error::WriteFileError(path_name.clone(), e))?;
        }
        Ok(())
    }
}
