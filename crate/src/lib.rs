use wasm_bindgen::prelude::*;

mod coords;
mod dom;
mod logic_paint;
mod modules;
mod upc;
mod utils;
mod viewport;
mod wgl2;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Other ideas:
// Pull buffer chunk data into an atlas. Rendering can then use an 'empty' texture to tile a single
//   quad for empty cells, and a pre-generated mesh that indexes the atlas (via UV coords) for
//   rendering. This will reduce the draw calls from 10k down to less than 10 at high zooms. It will
//   also reduce the sync-calls when updating run state.
// Auto-cycling: fundamental-clock until gate-state is stable between fundamental-clocks runs. Need
//  to put a max on this and warn the user because an oscillator would cause an infinite loop. Need
//  to do some perf testing, as it might be better to compare states only every N fundamental
//  clocks.
// Reporting on Cycles Per Second, Fundamental-Clocks per Cycle, and gate transitions per second.
