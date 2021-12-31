use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{
    dom::{DomIntervalEvent, ElementInputEvent},
    render_context::RenderContext,
    session::Session,
};

pub struct Viewport {
    pub session: Session,
    canvas: HtmlCanvasElement,
    render_context: RenderContext,
}

impl Viewport {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Viewport, JsValue> {
        let session = Session::new();
        let render_context = RenderContext::new(canvas.clone())?;

        Ok(Self {
            canvas,
            session,
            render_context,
        })
    }

    pub fn handle_element_input_event(&mut self, event: ElementInputEvent) {}

    pub fn handle_dom_interval_event(&mut self, event: DomIntervalEvent) {}

    pub fn set_session(&mut self, session: Session) {}

    pub fn draw(&mut self, time: f64) {
        // Maintain HTML Canvas size and context viewport.
        let (w, h) = (
            self.canvas.client_width() as u32,
            self.canvas.client_height() as u32,
        );
        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }
    }
}
