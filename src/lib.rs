use wasm_bindgen::prelude::*;

mod coords;
mod socket;
mod substrate;
mod upc;
mod utils;
mod viewport;
mod wgl2;
// mod modules;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
