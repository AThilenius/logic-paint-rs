use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
        texture::{Extent3d, TextureDimension, TextureFormat},
    },
};
use packed_struct::prelude::*;

pub const MOS_CANVAS_SIZE: usize = 64;

#[derive(PrimitiveEnum, Debug, Copy, Clone, PartialEq)]
pub enum SiLayerType {
    None = 0b00,
    PType = 0b01,
    NType = 0b10,
}

pub enum LayerState {
    Inactive = 0b00,
    Active = 0b01,
    Error = 0b10,
}

#[derive(PackedStruct, Debug, Copy, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0")]
pub struct MosCell {
    #[packed_field(bits = "0:1", ty = "enum")]
    pub lower_si_layer: SiLayerType,
    #[packed_field(bits = "2:3", ty = "enum")]
    pub upper_si_layer: SiLayerType,
    #[packed_field(bits = "4:5", ty = "enum")]
    pub lower_si_leyer_state: LayerState,
    #[packed_field(bits = "6:7", ty = "enum")]
    pub upper_si_leyer_state: LayerState,
    pub has_metal_layer: bool,
    pub has_via: bool,
    #[packed_field(bits = "10:11", ty = "enum")]
    pub metal_layer_state: LayerState,
}

/// Component that represents all Metal Oxide Silicon cells within an IC MOS canvas. All canvases
/// are fixed-size. Tiling can be added later if needed. You probably want to create a
/// MosCanvasBundle instead of adding this manually to an entity though.
pub struct MosCanvas {
    buffer: Vec<u8>,
    texture: Handle<Texture>,
}

impl MosCanvas {
    pub fn new(mut commands: Commands, textures: &mut Assets<Texture>) -> Self {
        let texture = textures.add(Texture::new_fill(
            Extent3d {
                width: MOS_CANVAS_SIZE,
                height: MOS_CANVAS_SIZE,
                depth: 1,
            },
            TextureDimension::D2,
            &[255, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
        ));

        let material = materials.add(MosMat {
            grid_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            grid_res: Vec2::new(w as f32, h as f32),
            texture: texture.clone(),
        });

        let quad = Mesh::from(shape::Quad {
            size: Vec2::new(400.0, 300.0),
            ..Default::default()
        });

        commands
            .spawn_bundle(MeshBundle {
                mesh: meshes.add(quad),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    pipeline_handle,
                )]),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            })
            .insert(material);
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());

        Self {
            buffer: vec![0u8; MOS_CANVAS_SIZE * MOS_CANVAS_SIZE * 4],
            texture,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> MosCell {
        let idx = (y * self.width + x) * 4;
        let bytes: [u8; 2] = [self.buffer[idx], self.buffer[idx + 1]];
        MosCell::unpack(&bytes).expect("failed to unpack cell data")
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: &MosCell) {
        let idx = (y * self.width + x) * 4;
        let packed: [u8; 2] = MosCell::pack(cell).expect("failed to pack cell data");
        self.buffer[idx] = packed[0];
        self.buffer[idx + 1] = packed[1];
    }
}

#[derive(Bundle, Default)]
pub struct MosCanvasBundle {
    pub mos_canvas: MosCanvas,
    pub mesh: Handle<Mesh>,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub main_pass: MainPass,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// The Bevy Plugin for rendering MosCanvas components.
pub struct MosRenderPlugin {}

struct MosRendererState {}

impl Plugin for MosRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = MosRendererState {};
        app.add_asset::<MosMat>()
            .insert_resource(state)
            .add_startup_system(setup.system())
            .add_system(update_grid_mat.system());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
struct MosMat {
    pub grid_color: Color,
    pub grid_res: Vec2,
    pub texture: Handle<Texture>,
}

fn update_mos_mat_texture(mut commands: Commands, mut materials: ResMut<Assets<MosMat>>) {}
