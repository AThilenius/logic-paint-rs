pub use bevy;

use bevy::prelude::*;
use input::{load_canvas_input, ActiveTools, CanvasInput, ToolType};

use crate::{
    canvas::{Canvas, SiLayerType},
    render::{CanvasRenderBundle, CanvasRenderPlugin, CellMaterial},
};

mod canvas;
mod input;
mod render;
mod utils;

pub struct CanvasShaderSource {
    pub vert: String,
    pub frag: String,
}

pub struct CommonPlugin {}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CanvasRenderPlugin {});
        app.insert_resource(ActiveTools::default());
        app.add_startup_system(setup.system());
        app.add_system(mouse_click.system());
        app.add_system(load_canvas_input.system());
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<CellMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: ResMut<AssetServer>,
) {
    asset_server.watch_for_changes().unwrap();

    commands
        .spawn_bundle(CanvasRenderBundle::new(
            &mut materials,
            &mut meshes,
            &mut textures,
            Transform::from_scale(Vec3::new(800.0, 800.0, 800.0)),
        ))
        .insert(Canvas::default())
        .insert(CanvasInput::default());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn mouse_click(
    mut canvas_query: Query<(&mut Canvas, &CanvasInput)>,
    active_tool: Res<ActiveTools>,
) {
    for (mut canvas, canvas_input) in canvas_query.iter_mut() {
        if !canvas_input.left_pressed && !canvas_input.right_pressed {
            return;
        }

        if let Some(pos) = canvas_input.mouse_position {
            update_cell(&mut canvas, &active_tool, pos, canvas_input.left_pressed);
        }
    }
}

fn update_cell(canvas: &mut Canvas, active_tool: &ActiveTools, pos: IVec2, left_click: bool) {
    match (active_tool.tool_type, left_click) {
        (ToolType::None, _) => {}
        (ToolType::PType, true) => {
            canvas.get_cell_mut(pos).lower_si_layer.layer_type = SiLayerType::PType
        }
        (ToolType::PType, false) => {
            canvas.get_cell_mut(pos).lower_si_layer.layer_type = SiLayerType::None
        }
        (ToolType::NType, true) => {
            canvas.get_cell_mut(pos).lower_si_layer.layer_type = SiLayerType::NType
        }
        (ToolType::NType, false) => {
            canvas.get_cell_mut(pos).lower_si_layer.layer_type = SiLayerType::None
        }
        (ToolType::Metal, true) => canvas.get_cell_mut(pos).has_metal = true,
        (ToolType::Metal, false) => canvas.get_cell_mut(pos).has_metal = false,
        (ToolType::Via, true) => canvas.get_cell_mut(pos).has_via = true,
        (ToolType::Via, false) => canvas.get_cell_mut(pos).has_via = false,
    };
}
