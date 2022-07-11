use std::collections::HashSet;

use glam::{IVec2, Vec2};

use crate::{coords::CellCoord, dom::RawInput, utils::range_iter, wgl2::Camera};

/// Tracks the overall input state of a viewport.
pub struct InputState {
    pub mouse_input: MouseInput,
    pub previous_mouse_input: MouseInput,

    pub keyboard_input: KeyboardInput,
    pub previous_keyboard_input: KeyboardInput,
}

/// Handles all the mouse input processing for use elsewhere like the Brush.
#[derive(Clone)]
pub struct MouseInput {
    pub primary: bool,
    pub secondary: bool,
    pub scroll_delta_y: f32,
    pub screen_point: Vec2,
    pub cell: CellCoord,
    pub drag: Option<Drag>,
}

#[derive(Clone, Default)]
pub struct KeyboardInput {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub keydown: HashSet<String>,
}

#[derive(Clone, Copy)]
pub struct Drag {
    pub start: CellCoord,
    pub initial_impulse_vertical: bool,
}

impl InputState {
    pub fn new() -> Self {
        let starting_mouse_input = MouseInput {
            primary: false,
            secondary: false,
            scroll_delta_y: 0.0,
            // Pick a point off-screen
            screen_point: Vec2::new(10_000.0, 10_000.0),
            // Pick a cell off-screen.
            cell: (1000, 1000).into(),
            drag: None,
        };

        Self {
            mouse_input: starting_mouse_input.clone(),
            previous_mouse_input: starting_mouse_input.clone(),
            keyboard_input: Default::default(),
            previous_keyboard_input: Default::default(),
        }
    }

    /// Updates self to reflect the `RawInput` event. Does not dispatch the event.
    pub fn handle_raw_input(&mut self, camera: &Camera, raw_input: &RawInput) {
        // Process mouse moving
        match raw_input {
            RawInput::MouseDown(e) | RawInput::MouseUp(e) | RawInput::MouseMove(e) => {
                // e.stop_propagation();
                // e.prevent_default();

                self.previous_mouse_input = self.mouse_input.clone();

                // Let's keep mouse buttons simple. I don't plan on ever using both bottoms. Leave
                // it to mean 'neither' is clicked. Aborting can be done with ESC.
                let left_mouse = e.buttons() & 1 != 0;
                let right_mouse = e.buttons() & 2 != 0;
                let shift = e.shift_key();

                let primary = left_mouse && !shift && !right_mouse;
                let secondary =
                    (right_mouse && !left_mouse) || (shift && (left_mouse ^ right_mouse));

                let screen_point = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
                let cell = camera.project_screen_point_to_cell(screen_point);

                let mut drag = self.previous_mouse_input.drag;

                // Handle drag start
                if (primary || secondary) && self.previous_mouse_input.drag.is_none() {
                    drag = Some(Drag {
                        start: cell,
                        initial_impulse_vertical: false,
                    });
                }

                // Handle drag ending
                if !primary && !secondary {
                    drag = None;
                }

                // Handle first cell that isn't drag_start while dragging (for initial impulse)
                if let Some(drag) = &mut drag {
                    if self.previous_mouse_input.cell == drag.start && cell != drag.start {
                        let dist = cell.0 - drag.start.0;
                        drag.initial_impulse_vertical = dist.x.abs() < dist.y.abs();
                    }
                }

                self.mouse_input = MouseInput {
                    primary,
                    secondary,
                    scroll_delta_y: 0.0,
                    screen_point,
                    cell,
                    drag,
                };
            }
            RawInput::KeyDown(e) => {
                // Don't preventDefault or stopPropagation otherwise vscode can't process keybinds.
                self.previous_keyboard_input = self.keyboard_input.clone();
                self.keyboard_input.ctrl = e.ctrl_key();
                self.keyboard_input.alt = e.alt_key();
                self.keyboard_input.shift = e.shift_key();
                self.keyboard_input.keydown.insert(e.code());
            }
            RawInput::KeyUp(e) => {
                self.previous_keyboard_input = self.keyboard_input.clone();
                self.keyboard_input.ctrl = e.ctrl_key();
                self.keyboard_input.alt = e.alt_key();
                self.keyboard_input.shift = e.shift_key();
                self.keyboard_input.keydown.remove(&e.code());
            }
            RawInput::MouseWheelEvent(e) => {
                self.previous_mouse_input = self.mouse_input.clone();

                // Let's keep mouse buttons simple. I don't plan on ever using both bottoms. Leave
                // it to mean 'neither' is clicked. Aborting can be done with ESC.
                let left_mouse = e.buttons() & 1 != 0;
                let right_mouse = e.buttons() & 2 != 0;
                let shift = e.shift_key();

                let primary = left_mouse && !shift && !right_mouse;
                let secondary =
                    (right_mouse && !left_mouse) || (shift && (left_mouse ^ right_mouse));

                let screen_point = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
                let cell = camera.project_screen_point_to_cell(screen_point);

                let scroll_delta_y = (e.delta_y() / 1000.0) as f32;

                self.mouse_input = MouseInput {
                    primary,
                    secondary,
                    scroll_delta_y,
                    screen_point,
                    cell,
                    drag: self.previous_mouse_input.drag,
                };
            }
        }
    }

    pub fn get_impulse_drag_path(&self) -> Vec<CellCoord> {
        let mut steps = vec![];
        let drag = self.previous_mouse_input.drag.or(self.mouse_input.drag);

        if let Some(drag) = drag {
            let start = drag.start.0;
            let end = self.mouse_input.cell.0;

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
        steps.push(self.mouse_input.cell);

        steps
    }
}
