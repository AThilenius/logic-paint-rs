use bevy::prelude::*;

use super::drawing::handle_canvas_input;

pub struct CanvasPlugin {
    // Config can go here.
}

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(handle_canvas_input.system());
    }
}
