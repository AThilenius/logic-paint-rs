use bevy_webgl2::WebGL2Plugin;
use common::{bevy::prelude::*, CanvasShaderSource, CommonPlugin};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    App::build()
        .insert_resource(CanvasShaderSource {
            vert: include_str!("shaders/cell.vert").to_string(),
            frag: include_str!("shaders/cell.frag").to_string(),
        })
        .insert_resource(WindowDescriptor {
            canvas: Some("#wasm-canvas".to_string()),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(CommonPlugin {})
        .add_plugin(WebGL2Plugin)
        .run();
}
