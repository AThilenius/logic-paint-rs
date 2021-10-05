use std::mem::forget;

use canvas_viewport::CanvasViewport;
use dom::{DomIntervalHooks, ElementEventHooks};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};

mod canvas_viewport;
mod dom;
mod input;
mod substrate_render_chunk;
mod utils;
mod wgl2;

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    let canvas_viewport = unwrap_or_log_and_return!(CanvasViewport::from_canvas(canvas.clone()));
    let dom_interval_hooks =
        unwrap_or_log_and_return!(DomIntervalHooks::new(canvas_viewport.clone()));
    let element_event_hooks = unwrap_or_log_and_return!(ElementEventHooks::new(
        canvas.dyn_into::<HtmlElement>().unwrap(),
        canvas_viewport.clone()
    ));

    forget(dom_interval_hooks);
    forget(element_event_hooks);
}
