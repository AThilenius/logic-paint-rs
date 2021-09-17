use bevy::{prelude::*, render::texture::Extent3d};

use crate::utils::HilbertIndexing;
use crate::{canvas::Canvas, render::CellMaterial};

pub fn render_canvas_to_texture(
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<CellMaterial>>,
    query: Query<(&Canvas, &Handle<CellMaterial>)>,
) {
    for (data, cell_material_handle) in query.iter() {
        let material = materials.get_mut(cell_material_handle).unwrap();
        let texture = textures.get_mut(material.cells_texture.clone()).unwrap();

        // See if the texture needs to be resized.
        // TODO: instead of resizing, using image tiling and multi-threading.
        if data.cells.size as u32 != texture.size.width
            || data.cells.size as u32 != texture.size.height
        {
            texture.resize(Extent3d {
                width: data.cells.size as u32,
                height: data.cells.size as u32,
                depth: 1,
            });
            material.grid_res = Vec2::new(data.cells.size as f32, data.cells.size as f32);
        }

        // Convert cells into bit-packed RGBA values and save them in the texture.
        let mut i = 0;

        // Textures are indexed with the upper-left being 0,0 but Canvas stores cells al the
        // lower-left being 0,0.
        for y in (0..data.cells.size).rev() {
            for x in 0..data.cells.size {
                data.cells
                    .get(IVec2::new(x as i32, y as i32))
                    .pack_into_4_bytes(&mut texture.data[i..(i + 4)]);

                i += 4;
            }
        }
    }
}
