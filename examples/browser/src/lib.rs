use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen(start)]
pub fn start() {
    web_sys::console::log_1(&format!("Hello world").into());
}
