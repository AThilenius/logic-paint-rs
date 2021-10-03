use canvas_viewport::CanvasViewport;
use wasm_bindgen::JsCast;

mod canvas_viewport;
mod substrate_render_chunk;
mod utils;
mod wgl2;

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    if let Err(e) = CanvasViewport::from_canvas(canvas) {
        log!("{:#?}", e);
    }
}
