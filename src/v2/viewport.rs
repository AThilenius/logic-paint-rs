use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{
    dom::{DomIntervalTarget, ElementEventTarget, ElementInputEvent},
    wgl2::Camera,
};

use super::{render_context::RenderContext, session::Session};

pub struct Viewport {
    camera: Camera,
    canvas: HtmlCanvasElement,
    session: Session,
    render_context: RenderContext,
}

impl Viewport {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Rc<RefCell<Viewport>>, JsValue> {
        let camera = Camera::new();
        let session = Session::new();
        let render_context = RenderContext::new(canvas.clone())?;

        let viewport = Rc::new(RefCell::new(Self {
            camera,
            canvas,
            session,
            render_context,
        }));

        Ok(viewport)
    }
}

impl DomIntervalTarget for Viewport {
    fn animation_frame(&mut self, time: f64) {
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

    fn simulate_step(&mut self) -> bool {
        false
    }
}

impl ElementEventTarget for Viewport {
    fn on_input_event(&mut self, event: ElementInputEvent) {
        self.camera.handle_mouse_event(event.clone());

        match &event {
            ElementInputEvent::MouseDown(e)
            | ElementInputEvent::MouseUp(e)
            | ElementInputEvent::MouseMove(e) => {
                // Mouse events to the Brush are only sent in the event handler if the button state changed
                // from the last event to now. Otherwise the latest event is simply deferred till render
                // time.
                // let pressed = (e.buttons() & 1 != 0, e.buttons() & 2 != 0);
                // if pressed != self.deferred_buttons_down {
                //     self.brush
                //         .handle_input_event(&self.camera, &self.ic, event.clone());
                //     self.deferred_buttons_down = pressed;
                // } else {
                //     self.deferred_mouse_event = Some(event.clone());
                // }
            }
            _ => {
                // All other events are sent right now.
                // self.brush
                //     .handle_input_event(&self.camera, &self.ic, event.clone());
            }
        };

        // match event {
        //     ElementInputEvent::KeyPressed(event) if event.code() == "KeyC" => {
        //         if let Some(_) = self.sim_state {
        //             self.sim_state = None;
        //         } else {
        //             self.sim_state = Some(self.ic.build_new_sim_state());
        //         }
        //     }
        //     _ => {}
        // }
    }
}
