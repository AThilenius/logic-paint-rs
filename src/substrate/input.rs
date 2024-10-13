use std::collections::HashSet;

use glam::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

use crate::{coords::CellCoord, wgl2::Camera};

pub enum RawInput {
    Mouse(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
}

/// Tracks the overall input state of a viewport.
#[derive(Default)]
#[wasm_bindgen]
pub struct InputState {
    pub primary: bool,
    pub secondary: bool,

    pub primary_clicked: bool,
    pub secondary_clicked: bool,

    pub scroll_delta_y: f32,
    pub screen_point: Vec2,
    pub cell: CellCoord,
    pub drag: Option<Drag>,

    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    #[wasm_bindgen(skip)]
    pub key_codes_down: HashSet<String>,
    #[wasm_bindgen(skip)]
    pub key_code_clicked: String,
    #[wasm_bindgen(skip)]
    pub key_clicked: String,
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct Drag {
    pub start: CellCoord,
    pub initial_impulse_vertical: bool,
}

impl InputState {
    /// Updates self to reflect the `RawInput` event. Events must be dispatched in lockstep, because
    /// the 'clicked' types only live a single invocation.
    pub fn handle_raw_input(&mut self, camera: &Camera, raw_input: &RawInput) {
        self.primary_clicked = false;
        self.secondary_clicked = false;
        self.key_code_clicked = "".to_owned();
        self.key_clicked = "".to_owned();
        self.scroll_delta_y = 0.0;

        // Process mouse moving
        match raw_input {
            RawInput::Mouse(e) => {
                let left_mouse = e.buttons() & 1 != 0;
                let right_mouse = e.buttons() & 2 != 0;
                let shift = e.shift_key();

                let primary = left_mouse && !shift && !right_mouse;
                let secondary =
                    (right_mouse && !left_mouse) || (shift && (left_mouse ^ right_mouse));

                self.primary_clicked = !self.primary && primary;
                self.secondary_clicked = !self.secondary && secondary;
                self.primary = primary;
                self.secondary = secondary;

                self.screen_point = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
                let new_cell = camera.project_screen_point_to_cell(self.screen_point);

                // Handle drag start
                if (self.primary || self.secondary) && self.drag.is_none() {
                    self.drag = Some(Drag {
                        start: new_cell,
                        initial_impulse_vertical: false,
                    });
                }

                // Handle drag ending
                if !self.primary && !self.secondary {
                    self.drag = None;
                }

                // Handle first cell that isn't drag_start while dragging (for initial impulse)
                if let Some(drag) = &mut self.drag {
                    if self.cell == drag.start && new_cell != drag.start {
                        let dist = new_cell.0 - drag.start.0;
                        drag.initial_impulse_vertical = dist.x.abs() < dist.y.abs();
                    }
                }

                self.cell = new_cell;
            }
            RawInput::KeyDown(e) => {
                self.ctrl = e.ctrl_key();
                self.alt = e.alt_key();
                self.shift = e.shift_key();

                self.key_codes_down.insert(e.code());

                if !e.repeat() {
                    self.key_code_clicked = e.code();
                    self.key_clicked = e.key();
                }
            }
            RawInput::KeyUp(e) => {
                self.ctrl = e.ctrl_key();
                self.alt = e.alt_key();
                self.shift = e.shift_key();

                self.key_codes_down.remove(&e.code());
            }
            RawInput::MouseWheelEvent(e) => {
                self.scroll_delta_y = (e.delta_y() / 1000.0) as f32;
            }
        }
    }
}
