extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate wasm_bin;

mod utils;

use std::path::PathBuf;

use utils::TestProject;
use wasm_bin::build;

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
            wasm-bindgen = "*"
        "#,
        )
        .file(
            "./src/main.rs",
            r#"
            #![feature(proc_macro, wasm_custom_section, wasm_import_module)]

            extern crate wasm_bindgen;

            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                fn alert(s: &str);
            }

            #[wasm_bindgen]
            extern {
                fn add(l: i32, r: i32) -> i32;
            }

            #[wasm_bindgen]
            pub extern "C" fn greet(name: &str) {
                alert(&format!("Hello, {}!", name));
            }
            
            pub fn main() {
                alert(&format!("{}", add(4, 7)));
            }
        "#,
        )
        .file(
            "./html/test_project_simple.html",
            r#"
            <html>
                <head>
                    <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
                    <script src='./test_project_simple.js'></script>
                    <script>
                        function add(l, r) {
                            return l + r;
                        }
                        window.addEventListener('load', function() {
                            wasm_bindgen("./test_project_simple_bg.wasm").then(function() {
                                wasm_bindgen.wasm.main();
                            });
                        }, false);
                    </script>
                </head>

                <body>
                    <h1>Welcome to this test project!</h1>
                </body>
            </html>"#,
        );

    let out_dir = PathBuf::from("./target/tests/build-project-simple");
    project.build(&out_dir).unwrap();
}
