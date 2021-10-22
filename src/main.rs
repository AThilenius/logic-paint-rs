use std::mem::forget;

use dom::{DomIntervalHooks, ElementEventHooks};
use viewport::Viewport;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};

mod brush;
mod dom;
mod sim;
mod substrate;
mod utils;
mod viewport;
mod wgl2;

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    let substrate_viewport = result_or_log_and_return!(Viewport::from_canvas(canvas.clone()));
    let dom_interval_hooks =
        result_or_log_and_return!(DomIntervalHooks::new(substrate_viewport.clone()));
    let element_event_hooks = result_or_log_and_return!(ElementEventHooks::new(
        canvas.dyn_into::<HtmlElement>().unwrap(),
        substrate_viewport.clone()
    ));

    forget(dom_interval_hooks);
    forget(element_event_hooks);
}
