use log::info;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlElement, MessageEvent, MessagePort};

mod gui;
mod utils;

#[wasm_bindgen]
extern "C" {
    pub type Plugin;

    #[wasm_bindgen(structural, method)]
    pub fn ping(this: &Plugin, msg: Vec<u8>) -> Vec<u8>;
}

#[wasm_bindgen]
pub fn register_plugin(plugin: Plugin) {
    let _ = plugin.ping(vec![42]);
}

#[wasm_bindgen]
pub struct Host {
    parent: HtmlElement,
    ui_ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Host {
    pub fn from_parent_element(parent: HtmlElement) -> Result<Host, JsValue> {
        let ui_ctx = spawn_ui_canvas(&parent.clone())?;
        Ok(Host { parent, ui_ctx })
    }

    pub fn frame(&mut self) {
        run(&self.parent, &self.ui_ctx.clone());
    }

    pub fn register_plugin_message_channel(&mut self, port: MessagePort) {
        let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();
            let data = data.as_string().unwrap();
            info!("Received message: {}", data);
        }) as Box<dyn FnMut(_)>);
        port.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback.forget();

        // Send ping to the plugin
        info!("Sending ping to plugin...");
        port.post_message(&JsValue::from_str("ping")).unwrap();
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

fn run(parent: &HtmlElement, ui_ctx: &CanvasRenderingContext2d) {
    let mut demo_ui = gui::demo_ui::DemoUi::new();

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
