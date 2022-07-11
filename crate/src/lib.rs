use wasm_bindgen::prelude::*;

mod blueprint;
mod brush;
mod buffer;
mod buffer_mask;
mod compiler;
mod coords;
mod dom;
mod execution_context;
mod input;
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

// Still to do:
// - Something with the focus is still fucked up.
// - Need to figure out how to represents a MOSFET with no gate connections. Deleting cells is a
//   real PITA when you can't leave MOSFETS in a broken state.

// The module.json file is always generated. I need this for having a sane way to define memory
// for both the micro-ops lookup, and main memory.

// ExecutionState has ticks and clock-cycles. A single tick is a single loop with change tracking.
// Ticking manually allows you to see the propagation of a signal through gates. It's only really
// useful for debugging. A clock-cycle is run by:
// - Each tick a did-change flag is kept. If that flag goes false, or MAX_TICKS_PER_CLOCK_EDGE is
//   reached, the clock is considered completed. The former is a short-circuit and will end
//   computation once the graph is in a stable state. The latter is for when the graph contains
//   an oscillator, and will never reach a steady state. Like a real clock, it will trigger again
//   after a certain number of serial transistor flips. This value should be runtime set.
// - Execution can be single-stepped, either per-tick, or per-clock. Otherwise the clock is ticked
//   at some floating-point multiple of seconds, up to and including infinity (as fast as you can
//   clock while still maintaining 60FPS).
// - Entering 'execution mode' puts you into manual-step initially.
// - Later on I would like to explore 'recording' execution state, and stepping forward/backward
//   by clock-cycles. This would be really useful for debugging when combined with a ring-buffer and
//   a 'breakpoint' module. It can answer the 'how the hell did I get here' question.

// Update is called on modules once per clock. Clock modules just flip their output each update, or
// by some divisor of the update rate.

// New terms:
// Fundamental Clock: the update loop in the ExecutionContext. It is the highest frequency a module
// can be updated at.
// Clock: module that clocks an I/O pin at an integer divisor of the Fundamental Clock.
