use bevy::{
    prelude::*,
    render::{
        mesh::shape,
        pipeline::RenderPipeline,
        render_graph::base::MainPass,
        texture::{Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat},
    },
};

use crate::{
    canvas::DEFAULT_CANVAS_SIZE, render::pipeline::CELL_PIPELINE_HANDLE, render::CellMaterial,
};

#[derive(Bundle)]
pub struct CanvasRenderBundle {
    pub draw: Draw,
    pub global_transform: GlobalTransform,
    pub main_pass: MainPass,
    pub material: Handle<CellMaterial>,
    pub mesh: Handle<Mesh>,
    pub render_pipelines: RenderPipelines,
    pub texture: Handle<Texture>,
    pub transform: Transform,
    pub visible: Visible,
}

impl CanvasRenderBundle {
    pub fn new(
        materials: &mut Assets<CellMaterial>,
        meshes: &mut Assets<Mesh>,
        textures: &mut Assets<Texture>,
        transform: Transform,
    ) -> Self {
        // The texture is unsigned, un-normalized 8-bit, so min and mag filters have to be Nearest.
        let mut texture = Texture {
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            sampler: SamplerDescriptor {
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        };
        texture.resize(Extent3d {
            width: DEFAULT_CANVAS_SIZE as u32,
            height: DEFAULT_CANVAS_SIZE as u32,
            depth: 1,
        });

        let pixel = [1, 0, 0, 1];
        for current_pixel in texture.data.chunks_exact_mut(pixel.len()) {
            current_pixel.copy_from_slice(&pixel);
        }

        let texture = textures.add(texture);

        let material = materials.add(CellMaterial {
            grid_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            grid_res: Vec2::new(DEFAULT_CANVAS_SIZE as f32, DEFAULT_CANVAS_SIZE as f32),
            n_color: Color::rgba(0.0, 0.5, 0.0, 1.0),
            p_color: Color::rgba(0.0, 0.0, 0.5, 1.0),
            texture: texture.clone(),
        });

        let quad = Mesh::from(shape::Quad {
            size: Vec2::new(1.0, 1.0),
            ..Default::default()
        });

        let mesh = meshes.add(quad);
        let render_pipelines = RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            CELL_PIPELINE_HANDLE.typed(),
        )]);

        Self {
            draw: Default::default(),
            global_transform: Default::default(),
            main_pass: Default::default(),
            material,
            mesh,
            render_pipelines,
            texture: texture.clone(),
            transform,
            visible: Default::default(),
        }
    }
}
