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
