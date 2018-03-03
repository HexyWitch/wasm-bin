#![feature(proc_macro)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[no_mangle]
pub extern "C" fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
