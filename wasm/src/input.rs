use web_sys::MouseEvent;

use crate::log;

/// Processed input for a canvas (js input handlers are registered in canvas_viewport.rs).
#[derive(Default)]
pub struct Input {
    pending_mouse_events: Vec<RawMouseEvent>,
}

impl Input {
    pub fn enqueue_raw(&mut self, raw_mouse_event: RawMouseEvent) {
        log!("Enqueue: {:#?}", &raw_mouse_event);
        self.pending_mouse_events.push(raw_mouse_event);
    }
}

#[derive(Clone, Debug)]
pub enum RawMouseEvent {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
}
