use std::{collections::HashSet, i32};

use glam::{IVec2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

use crate::{coords::CellCoord, utils::range_iter, wgl2::Camera};

/// Every input the core is capable of processing.
pub enum RawInput {
    Mouse(MouseEvent),
    MouseWheel(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    MousePresence(bool),
}

/// Thin wrapper around the discriminated-union RawInput
#[wasm_bindgen]
pub struct JsInputEvent {
    pub(crate) raw_input: RawInput,
}

/// Tracks the overall input state of a viewport.
#[derive(Default)]
pub struct InputState {
    pub active: bool,

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
    pub key_codes_down: HashSet<String>,
    pub key_code_clicked: String,
    pub key_clicked: String,
}

/// Every output (apart from rendering) the core is capably of communicating.
/// This state is only valid between JsInputEvent processing, and should be
/// treated as stale once a subsequent event has been processed.
#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct OutputState {
    /// The CSS `cursor` value that corresponds with the current state of the Viewport.
    pub viewport_mouse_cursor: String,

    /// Set to true when the `competed_buffer` has a pending change, and has not yet been fetched
    /// with `clone_competed_buffer`.
    pub completed_buffer_dirty: bool,

    /// What named registers are dirty and have not yet been fetched with `clone_named_register`.
    /// These should be saved somewhere locally-persistent, like localStorage.
    pub named_registers_dirty: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct Drag {
    pub start: CellCoord,
    pub initial_impulse_vertical: bool,
}

#[wasm_bindgen]
impl JsInputEvent {
    pub fn from_key_down(event: KeyboardEvent) -> Self {
        Self {
            raw_input: RawInput::KeyDown(event),
        }
    }

    pub fn from_key_up(event: KeyboardEvent) -> Self {
        Self {
            raw_input: RawInput::KeyUp(event),
        }
    }

    pub fn from_mouse(event: MouseEvent) -> Self {
        Self {
            raw_input: RawInput::Mouse(event),
        }
    }

    pub fn from_mouse_presence(present: bool) -> Self {
        Self {
            raw_input: RawInput::MousePresence(present),
        }
    }

    pub fn from_wheel(event: WheelEvent) -> Self {
        Self {
            raw_input: RawInput::MouseWheel(event),
        }
    }
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
            RawInput::MousePresence(present) => {
                self.active = *present;
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
            RawInput::MouseWheel(e) => {
                self.scroll_delta_y = (e.delta_y() / 1000.0) as f32;
            }
        }
    }

    pub fn get_impulse_drag_path(&self) -> Vec<CellCoord> {
        let mut steps = vec![];

        if let Some(drag) = &self.drag {
            let start = drag.start.0;
            let end = self.cell.0;

            if drag.initial_impulse_vertical {
                // Draw Y first, then X.
                for y in range_iter(start.y, end.y) {
                    steps.push(CellCoord(IVec2::new(start.x, y)));
                }
                for x in range_iter(start.x, end.x) {
                    steps.push(CellCoord(IVec2::new(x, end.y)));
                }
            } else {
                // Draw X first, then Y.
                for x in range_iter(start.x, end.x) {
                    steps.push(CellCoord(IVec2::new(x, start.y)));
                }
                for y in range_iter(start.y, end.y) {
                    steps.push(CellCoord(IVec2::new(end.x, y)));
                }
            }
        }

        // The last point will be skipped because range_iter is non-inclusive of end point.
        steps.push(self.cell);

        steps
    }
}
