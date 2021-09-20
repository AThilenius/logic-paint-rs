use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};

use super::{CanvasRenderBundle, CANVAS_DEPTH, CELL_CHUNK_SIZE};
use crate::{canvas::Canvas, render::rasterizer::render_canvas_to_texture};

pub(crate) const CELL_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0x5a7cd988468a9060);

const CELL_MATERIAL: &str = "cell_material";

pub struct CanvasRenderPlugin {}

impl Plugin for CanvasRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<CellMaterial>()
            .add_startup_system(setup.system())
            .add_system(auto_attach_render_bundle.system())
            .add_system(render_canvas_to_texture.system());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
pub struct CellMaterial {
    pub grid_color: Color,
    pub grid_res: Vec2,
    pub n_color: Color,
    pub p_color: Color,
    pub cells_texture: Handle<Texture>,
}

impl CellMaterial {
    pub fn standard(cells_texture: Handle<Texture>) -> Self {
        Self {
            grid_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            grid_res: Vec2::new(CELL_CHUNK_SIZE as f32, CELL_CHUNK_SIZE as f32),
            n_color: Color::rgba(0.0, 0.5, 0.0, 1.0),
            p_color: Color::rgba(0.0, 0.0, 0.5, 1.0),
            cells_texture,
        }
    }
}

fn setup(
    mut graph: ResMut<RenderGraph>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    asset_server: ResMut<AssetServer>,
    // mut shaders: ResMut<Assets<Shader>>,
    // shader_source: Res<CanvasShaderSource>,
) {
    graph.add_system_node(
        CELL_MATERIAL,
        AssetRenderResourcesNode::<CellMaterial>::new(false),
    );

    graph
        .add_node_edge(CELL_MATERIAL, base::node::MAIN_PASS)
        .unwrap();

    // pipelines.set_untracked(
    //     CELL_PIPELINE_HANDLE,
    //     PipelineDescriptor::default_config(ShaderStages {
    //         vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, &shader_source.vert)),
    //         fragment: Some(
    //             shaders.add(Shader::from_glsl(ShaderStage::Fragment, &shader_source.frag)),
    //         ),
    //     }),
    // );

    pipelines.set_untracked(
        CELL_PIPELINE_HANDLE,
        PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/cell.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/cell.frag")),
        }),
    );
}

fn auto_attach_render_bundle(
    mut commands: Commands,
    mut materials: ResMut<Assets<CellMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Texture>>,
    query: Query<Entity, (With<Canvas>, Without<Handle<Mesh>>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(CanvasRenderBundle::new(
                &mut materials,
                &mut meshes,
                &mut textures,
                Transform::from_translation(Vec3::new(0.0, 0.0, CANVAS_DEPTH)),
            ));
    }
}
