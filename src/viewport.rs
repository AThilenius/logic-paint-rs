use glam::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement};

use crate::{
    dom::{DomIntervalEvent, ElementInputEvent},
    log,
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

    pub fn handle_dom_interval_event(&mut self, event: DomIntervalEvent) {
        match event {
            DomIntervalEvent::RequestAnimationFrame(time) => {
                self.draw(time);
            }
        }
    }

    pub fn set_session(&mut self, session: Session) {
        // TODO: Is this right?
        self.session = session;
    }

    fn draw(&mut self, time: f64) {
        // Maintain HTML Canvas size and context viewport.
        let w = self.canvas.client_width() as u32;
        let h = self.canvas.client_height() as u32;

        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }

        self.session.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );

        self.render_context.draw(time, &self.session).unwrap_throw();
    }
}
