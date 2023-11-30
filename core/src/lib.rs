use log::info;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlElement};

use crate::gui::types::Point;

mod gui;
mod utils;

#[wasm_bindgen]
pub struct Host {
    parent: HtmlElement,
    ui_ctx: CanvasRenderingContext2d,
    demo_ui: gui::demo_ui::DemoUi,
}

#[wasm_bindgen]
impl Host {
    pub fn from_parent_element(parent: HtmlElement) -> Result<Host, JsValue> {
        let ui_ctx = spawn_ui_canvas(&parent.clone())?;
        Ok(Host {
            parent,
            ui_ctx,
            demo_ui: gui::demo_ui::DemoUi::new(),
        })
    }

    pub fn frame(&mut self) {
        run(&self.parent, &self.ui_ctx, &mut self.demo_ui);
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.demo_ui.root.test_dispatch_move(Point {
            left: x as f32,
            top: y as f32,
        });
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    console_log::init().expect("could not initialize logger");
}

fn spawn_ui_canvas(parent: &web_sys::HtmlElement) -> Result<CanvasRenderingContext2d, JsValue> {
    let canvas = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.create_element("canvas").ok())
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| JsValue::from_str("Failed to create canvas"))?;

    // Register a mouse-move handler for the canvas element
    let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x();
        let y = event.offset_y();
        info!("mouse move: {}, {}", x, y);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();

    parent.append_child(&canvas)?;

    canvas.set_width(parent.client_width() as u32);
    canvas.set_height(parent.client_height() as u32);

    let style = canvas.style();
    style.set_property("position", "absolute")?;
    style.set_property("inset", "0")?;

    let context = canvas
        .get_context("2d")?
        .and_then(|ctx| ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?;

    Ok(context)
}

fn run(
    parent: &HtmlElement,
    ui_ctx: &CanvasRenderingContext2d,
    demo_ui: &mut gui::demo_ui::DemoUi,
) {
    let canvas = ui_ctx.canvas().unwrap();

    // Resize the canvas to match the parent size, if needed.
    let parent_width = parent.client_width() as u32;
    let parent_height = parent.client_height() as u32;
    if canvas.width() != parent_width || canvas.height() != parent_height {
        canvas.set_width(parent_width);
        canvas.set_height(parent_height);
    }

    ui_ctx.set_font("16px Courier New");
    ui_ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

    demo_ui.draw(&ui_ctx);
}
