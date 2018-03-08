extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;

use std::path::Path;

use wasm_build_support::build;

#[test]
fn build_project_simple() {
    let project_dir = Path::new("./tests/test-project-simple");
    std::env::set_current_dir(project_dir).expect("Error setting working directory");

    let mut options = build::Options::default();
    options.manifest_path = Some("_Cargo.toml".to_string());
    build::build(&options).unwrap();
}
