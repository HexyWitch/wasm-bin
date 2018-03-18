extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_build_support;
extern crate wasm_build_test;

use std::path::PathBuf;

use wasm_build_support::build;
use wasm_build_test::TestProject;

#[test]
fn build_project_simple() {
    let build_options = build::Options::default();
    let mut project = TestProject::new(build_options);
    project
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "test_project_simple"
            version = "0.0.1"

            [workspace]

            [dependencies]
            wasm-bindgen = { git = 'https://github.com/alexcrichton/wasm-bindgen' }
        "#,
        )
        .file(
            "./src/main.rs",
            r#"
            #![feature(proc_macro)]

            extern crate wasm_bindgen;

            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                fn alert(s: &str);
            }

            #[wasm_bindgen(module  = "./add")]
            extern {
                fn add(l: i32, r: i32) -> i32;
            }

            #[wasm_bindgen]
            pub extern "C" fn greet(name: &str) {
                alert(&format!("Hello, {}!", name));
            }

            #[wasm_bindgen]
            pub fn web_main() {
                main();
            }

            pub fn main() {
                alert(&format!("{}", add(4, 7)));
            }
        "#,
        )
        .file(
            "./js/add.js",
            "
            export function add(l, r) {
                return l + r;
            }
        ",
        )
        .file(
            "./html/test_project_simple.html",
            r#"
            <html>
                <head>
                    <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
                </head>

                <body>
                    <h1>Welcome to this test project!</h1>
                    <script src='./test_project_simple.js'></script>
                </body>
            </html>"#,
        );

    let out_dir = PathBuf::from("./target/tests/build-project-simple");
    project.build(&out_dir).unwrap();
}
