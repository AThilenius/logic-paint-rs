use glam::{IVec2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

use crate::{coords::CellCoord, utils::range_iter, wgl2::Camera};

#[wasm_bindgen]
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub struct BoolState {
    /// The key was just clicked this dispatch.
    pub clicked: bool,

    /// The key is being held down. Can be true when `clicked` is true.
    pub down: bool,

    /// The key was just released this dispatch.
    pub released: bool,
}

impl BoolState {
    /// Steps the boolean state forward if it's in a single-frame state (clicked or released). Has
    /// no effect if the state is steady (inactive or held)
    pub fn tick(&mut self) {
        if self.clicked {
            // State continues to be true
            *self = Self {
                clicked: false,
                down: true,
                released: false,
            }
        } else if self.released {
            // State finished release
            *self = Self {
                clicked: false,
                down: false,
                released: false,
            }
        }
    }

    /// Transisition the boolean state forward, given the new input. Steps to/from both
    /// single-frame states and steady states.
    pub fn transition(self, new_state: bool) -> Self {
        if new_state {
            if !self.down {
                // State is first true
                Self {
                    clicked: true,
                    down: true,
                    released: false,
                }
            } else {
                // State continues to be true
                Self {
                    clicked: false,
                    down: true,
                    released: false,
                }
            }
        } else {
            if self.down {
                // State is first false
                Self {
                    clicked: false,
                    down: false,
                    released: true,
                }
            } else {
                // State continues to be false
                Self {
                    clicked: false,
                    down: false,
                    released: false,
                }
            }
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct KeyState {
    pub key_code: String,
    pub key: String,
    pub state: BoolState,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub start: CellCoord,
    pub initial_impulse_vertical: bool,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Default)]
pub struct IoState {
    pub hovered: BoolState,

    pub primary: BoolState,
    pub secondary: BoolState,
    pub drag: Option<Drag>,
    pub keys: Vec<KeyState>,
    pub screen_point: Vec2,
    pub cell: CellCoord,
    pub scroll_delta_y: f32,
}

#[wasm_bindgen]
impl IoState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn event_key_down(&mut self, e: KeyboardEvent) {
        let key_state = KeyState {
            key: e.key(),
            key_code: e.code(),
            state: self.get_key_code(&e.code()).transition(true),
        };

        self.keys.retain(|key| key.key_code != e.code());
        self.tick();

        self.keys.push(key_state);
    }

    pub fn event_key_up(&mut self, e: KeyboardEvent) {
        let key_state = KeyState {
            key: e.key(),
            key_code: e.code(),
            state: self.get_key_code(&e.code()).transition(false),
        };

        self.keys.retain(|key| key.key_code != e.code());
        self.tick();

        self.keys.push(key_state);
    }

    pub fn event_mouse(&mut self, e: MouseEvent, camera: &Camera) {
        let primary = self.primary.transition(e.buttons() & 1 != 0);
        let secondary = self.secondary.transition(e.buttons() & 2 != 0);

        let new_cell = camera.project_screen_point_to_cell(self.screen_point);

        self.tick();

        // Handle drag start
        if (primary.clicked || secondary.clicked) && self.drag.is_none() {
            self.drag = Some(Drag {
                start: new_cell,
                initial_impulse_vertical: false,
            });
        }

        // Handle drag ending
        if !primary.down && !secondary.down {
            self.drag = None;
        }

        // Handle first cell that isn't drag_start while dragging (for initial impulse)
        if let Some(drag) = &mut self.drag {
            if self.cell == drag.start && new_cell != drag.start {
                let dist = new_cell.0 - drag.start.0;
                drag.initial_impulse_vertical = dist.x.abs() < dist.y.abs();
            }
        }

        self.primary = primary;
        self.secondary = secondary;
        self.screen_point = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
        self.cell = new_cell;
    }

    pub fn event_mouse_presence(&mut self, presence: bool) {
        let hovered = self.hovered.transition(presence);
        self.tick();
        self.hovered = hovered;
    }

    pub fn event_wheel(&mut self, e: WheelEvent) {
        self.tick();
        self.scroll_delta_y = (e.delta_y() / 1000.0) as f32;
    }

    fn tick(&mut self) {
        // Tick all input boolean states
        self.hovered.tick();
        self.primary.tick();
        self.secondary.tick();

        // Tick all keys
        for key in &mut self.keys {
            key.state.tick();
        }

        // And only retain active
        self.keys.retain(|k| k.state != Default::default());

        // Reset scroll
        self.scroll_delta_y = 0.0;

        // Reset output state
        // self.cursor = "default".to_string();
        // self.buffer_persist = None;
        // self.tools_persist.clear();
    }

    pub fn get_key(&self, key: &str) -> BoolState {
        for key_state in &self.keys {
            if key_state.key == key {
                return key_state.state;
            }
        }

        Default::default()
    }

    pub fn get_key_code(&self, key_code: &str) -> BoolState {
        for key_state in &self.keys {
            if key_state.key_code == key_code {
                return key_state.state;
            }
        }

        Default::default()
    }

    pub fn get_drag_path(&self) -> Vec<CellCoord> {
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
