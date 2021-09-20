use bevy::prelude::*;
use canvas::CanvasBundle;
use render::LABEL_DEPTH;

use crate::{canvas::CanvasPlugin, render::CanvasRenderPlugin};

// Mods
pub mod canvas;
mod render;
mod sim;
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

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    commands.spawn_bundle(CanvasBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // DEV
    let font = asset_server.load("fonts/roboto-mono-v13-latin-regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 16.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Left,
    };

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section("translation", text_style.clone(), text_alignment),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, LABEL_DEPTH)),
        ..Default::default()
    });
}
