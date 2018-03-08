extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate parity_wasm;

mod util;
pub mod cargo;
pub mod bindgen;
pub mod webpack;
pub mod build;
pub mod wasm_post;
