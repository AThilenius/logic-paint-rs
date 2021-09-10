use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::{Extent3d, TextureDimension, TextureFormat},
    },
};

use super::{sprite_texture_data::SPRITE_SHEET_TEXTURE_DATA, CanvasRenderBundle};
use crate::{
    canvas::{CanvasData, DEFAULT_CANVAS_SIZE},
    render::rasterizer::render_canvas_to_texture,
};

pub(crate) const CELL_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0x5a7cd988468a9060);

const CELL_MATERIAL: &str = "cell_material";

pub struct CanvasRenderPlugin {}

#[derive(Default)]
struct CanvasRenderState {
    cell_atlas_texture: Handle<Texture>,
}

impl Plugin for CanvasRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<CellMaterial>()
            .insert_resource(CanvasRenderState::default())
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
    pub atlas_texture: Handle<Texture>,
}

impl CellMaterial {
    pub fn standard(cells_texture: Handle<Texture>, atlas_texture: Handle<Texture>) -> Self {
        Self {
            grid_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            grid_res: Vec2::new(DEFAULT_CANVAS_SIZE as f32, DEFAULT_CANVAS_SIZE as f32),
            n_color: Color::rgba(0.0, 0.5, 0.0, 1.0),
            p_color: Color::rgba(0.0, 0.0, 0.5, 1.0),
            cells_texture,
            atlas_texture,
        }
    }
}

fn setup(
    mut render_state: ResMut<CanvasRenderState>,
    mut graph: ResMut<RenderGraph>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: ResMut<AssetServer>,
    // mut shaders: ResMut<Assets<Shader>>,
    // shader_source: Res<CanvasShaderSource>,
) {
    // Sprite sheet texture for cell rendering.
    let mut texture = Texture {
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        // sampler: SamplerDescriptor {
        //     min_filter: FilterMode::Nearest,
        //     mag_filter: FilterMode::Nearest,
        //     ..Default::default()
        // },
        ..Default::default()
    };
    texture.resize(Extent3d {
        width: 32,
        height: 32,
        depth: 1,
    });

    texture.data = SPRITE_SHEET_TEXTURE_DATA.to_vec();
    render_state.cell_atlas_texture = textures.add(texture);

    // Setup the material and shaders.
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
    render_state: Res<CanvasRenderState>,
    query: Query<Entity, (With<CanvasData>, Without<Handle<Mesh>>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(CanvasRenderBundle::new(
                &mut materials,
                &mut meshes,
                &mut textures,
                render_state.cell_atlas_texture.clone(),
                Transform::from_scale(Vec3::new(800.0, 800.0, 800.0)),
            ));
    }
}
