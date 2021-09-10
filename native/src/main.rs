use common::{bevy::prelude::*, CanvasShaderSource, CommonPlugin};

fn main() {
    App::build()
        .insert_resource(CanvasShaderSource {
            vert: include_str!("shaders/cell.vert").to_string(),
            frag: include_str!("shaders/cell.frag").to_string(),
        })
        // .insert_resource(AssetServerSettings {
        //     asset_folder: "../assets".to_string(),
        // })
        .add_plugins(DefaultPlugins)
        .add_plugin(CommonPlugin {})
        .run();
}
