use bevy::prelude::*;
use input::{load_canvas_input, CanvasInput};

use crate::{
    canvas::{Canvas, SiLayerType},
    render::{CanvasRenderBundle, CanvasRenderPlugin, CellMaterial},
};

mod canvas;
mod input;
mod render;
mod utils;

/// This example illustrates how to create a custom material asset and a shader that uses that
/// material
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CanvasRenderPlugin {})
        .add_startup_system(setup.system())
        .add_system(mouse_click.system())
        .add_system(load_canvas_input.system())
        .run();
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
            Transform::from_scale(Vec3::new(400.0, 400.0, 400.0)),
        ))
        .insert(Canvas::default())
        .insert(CanvasInput::default());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn mouse_click(mut canvas_query: Query<(&mut Canvas, &CanvasInput)>) {
    for (mut canvas, canvas_input) in canvas_query.iter_mut() {
        if !canvas_input.left_pressed {
            return;
        }

        for pos in canvas_input.mouse_moved.iter() {
            canvas.get_cell_mut(*pos).lower_si_layer.layer_type = SiLayerType::NType;
        }
    }
}
