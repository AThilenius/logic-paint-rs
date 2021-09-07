use bevy::{
    prelude::*,
    render::{
        mesh::shape,
        pipeline::RenderPipeline,
        render_graph::base::MainPass,
        texture::{Extent3d, TextureDimension, TextureFormat},
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
        let texture = textures.add(Texture::new_fill(
            Extent3d {
                width: DEFAULT_CANVAS_SIZE as u32,
                height: DEFAULT_CANVAS_SIZE as u32,
                depth: 1,
            },
            TextureDimension::D2,
            &[255, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
        ));

        let material = materials.add(CellMaterial {
            grid_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            grid_res: Vec2::new(DEFAULT_CANVAS_SIZE as f32, DEFAULT_CANVAS_SIZE as f32),
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
