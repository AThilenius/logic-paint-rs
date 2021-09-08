use bevy::prelude::*;

use crate::render::{
    pipeline::{add_cell_graph, CellMaterial},
    rasterizer::render_canvas_to_texture,
};

pub struct CanvasRenderPlugin {
    // Config can go here.
}

struct CanvasRenderState {
    // State can go here.
}

impl Plugin for CanvasRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = CanvasRenderState {};
        app.add_asset::<CellMaterial>()
            .insert_resource(state)
            .add_startup_system(add_cell_graph.system())
            .add_system(render_canvas_to_texture.system());
    }
}
