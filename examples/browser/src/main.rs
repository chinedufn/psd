use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen(start)]
fn main() {
    web_sys::console::log_1(&format!("Hello world").into());
}
