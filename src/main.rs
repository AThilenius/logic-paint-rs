use std::mem::forget;

use dom::{DomIntervalHooks, ElementEventHooks};
use miniz_oxide::inflate::decompress_to_vec;
use viewport::Viewport;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};

use crate::substrate::deserialize_ic;

mod brush;
mod dom;
mod substrate;
mod utils;
mod viewport;
mod wgl2;

// Keybinds (WIP):
// i - Insert mode: allows drawing
// v

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    let substrate_viewport = result_or_log_and_return!(Viewport::from_canvas(canvas.clone()));

    // Try to load from local storage
    if let Ok(Some(compressed_b64)) = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item(&"logic-paint-ic")
    {
        if let Ok(compressed_bytes) = base64::decode(compressed_b64) {
            if let Ok(bytes) = decompress_to_vec(&compressed_bytes) {
                let ic = deserialize_ic(&bytes);
                substrate_viewport.borrow_mut().set_ic(ic);
            }
        }
    }

    let dom_interval_hooks =
        result_or_log_and_return!(DomIntervalHooks::new(substrate_viewport.clone()));
    let element_event_hooks = result_or_log_and_return!(ElementEventHooks::new(
        canvas.dyn_into::<HtmlElement>().unwrap(),
        substrate_viewport.clone()
    ));

    forget(dom_interval_hooks);
    forget(element_event_hooks);
}
