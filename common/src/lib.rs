use bevy::prelude::*;
use canvas::CanvasBundle;

use crate::{
    canvas::CanvasPlugin,
    render::{CanvasRenderBundle, CanvasRenderPlugin, CellMaterial},
};

// Mods
pub mod canvas;
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
        app.add_plugin(CanvasPlugin {});
        app.add_startup_system(setup.system());
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

    commands.spawn_bundle(CanvasBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
