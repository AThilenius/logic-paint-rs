use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

pub(crate) const CELL_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 4930123987508190269);

const CELL_MATERIAL: &str = "cell_material";

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
pub struct CellMaterial {
    pub grid_color: Color,
    pub grid_res: Vec2,
    pub texture: Handle<Texture>,
}

pub(crate) fn add_cell_graph(
    mut graph: ResMut<RenderGraph>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    graph.add_system_node(
        CELL_MATERIAL,
        AssetRenderResourcesNode::<CellMaterial>::new(false),
    );

    graph
        .add_node_edge(CELL_MATERIAL, base::node::MAIN_PASS)
        .unwrap();

    pipelines.set_untracked(
        CELL_PIPELINE_HANDLE,
        PipelineDescriptor::default_config(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("cell.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("cell.frag"),
            ))),
        }),
    );
}
