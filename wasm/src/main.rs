use std::{cell::RefCell, rc::Rc};

use canvas_viewport::CanvasViewport;
use wasm_bindgen::{prelude::Closure, JsCast};

mod canvas_viewport;
mod substrate_render_chunk;
mod utils;
mod wgl2;

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let canvas_viewport = CanvasViewport::from_canvas(canvas);

    if let Err(e) = canvas_viewport {
        log!("{:#?}", e);
        return;
    }
    let canvas_viewport = canvas_viewport.unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        canvas_viewport.draw();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
