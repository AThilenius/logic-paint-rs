use bevy::prelude::*;
use input::{load_canvas_input, ActiveTools, CanvasInput};

use crate::{
    canvas::{CanvasData, CanvasPlugin, SiLayer},
    render::{CanvasRenderBundle, CanvasRenderPlugin, CellMaterial},
};

// Mods
pub mod canvas;
mod input;
mod render;
mod utils;

// Re-exports
pub use bevy;

pub struct CanvasShaderSource {
    pub vert: String,
    pub frag: String,
}

pub struct CommonPlugin {}

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CanvasRenderPlugin {});
        app.insert_resource(ActiveTools::default());
        app.add_plugin(CanvasPlugin {});
        app.add_startup_system(setup.system());
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
        .insert(CanvasData::default())
        .insert(CanvasInput::default());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
