#![allow(
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables,
    mismatched_lifetime_syntaxes,
    clippy::assign_op_pattern,
    clippy::clone_on_copy,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::derivable_impls,
    clippy::explicit_auto_deref,
    clippy::filter_next,
    clippy::identity_op,
    clippy::len_zero,
    clippy::manual_is_multiple_of,
    clippy::needless_borrow,
    clippy::needless_return,
    clippy::single_match,
    clippy::upper_case_acronyms,
    clippy::useless_conversion,
    clippy::wrong_self_convention
)]

use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_thread as thread;

mod coords;
mod module;
mod project;
mod render;
mod substrate;
mod tools;
mod upc;
mod utils;
mod viewport;
mod wgl2;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn run_thread_test() {
    for _ in 0..2 {
        thread::spawn(|| {
            for i in 1..3 {
                log!(
                    "hi number {} from the spawned thread {:?}!",
                    i,
                    thread::current().id()
                );
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    for i in 1..3 {
        log!(
            "hi number {} from the main thread {:?}!",
            i,
            thread::current().id()
        );
    }
}
