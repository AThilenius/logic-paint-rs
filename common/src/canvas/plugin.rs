use bevy::prelude::*;

use super::{
    drawing::handle_canvas_input,
    input::{load_canvas_input, ActiveTools},
};

pub struct CanvasPlugin {
    // Config can go here.
}

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ActiveTools::default());
        app.add_system(load_canvas_input.system());
        app.add_system(handle_canvas_input.system());
    }
}
