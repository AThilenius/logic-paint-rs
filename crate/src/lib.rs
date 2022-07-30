use wasm_bindgen::prelude::*;

mod coords;
mod dom;
mod logic_paint;
mod modules;
mod substrate;
mod upc;
mod utils;
mod viewport;
mod wgl2;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// The module.json file is always generated. I need this for having a sane way to define memory
// for both the micro-ops lookup, and main memory.
