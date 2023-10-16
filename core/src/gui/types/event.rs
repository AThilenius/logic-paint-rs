use std::collections::HashSet;

use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseEvent {
        cursor: Vec2,
        primary: bool,
        secondary: bool,
        primary_clicked: bool,
        secondary_clicked: bool,
        scroll_delta_y: f32,

        // Provided as a convenience.
        ctrl: bool,
        shift: bool,
        alt: bool,
    },
    KeyboardEvent {
        key_code_down: HashSet<String>,
        key_code_clicked: String,
        key_clicked: String,
    },
    Focus {
        mouse_entered: bool,
        mouse_exited: bool,
        focused: bool,
        blurred: bool,
    },
}
