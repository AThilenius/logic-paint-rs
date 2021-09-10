use bevy::{prelude::*, render::texture::Extent3d};

use crate::{canvas::CanvasData, render::CellMaterial};

pub fn render_canvas_to_texture(
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<CellMaterial>>,
    query: Query<(&CanvasData, &Handle<CellMaterial>)>,
) {
    for (canvas, cell_material_handle) in query.iter() {
        let material = materials.get_mut(cell_material_handle).unwrap();
        let texture = textures.get_mut(material.cells_texture.clone()).unwrap();

        // See if the texture needs to be resized.
        // TODO: instead of resizing, using image tiling and multi-threading.
        if canvas.size != texture.size.width || canvas.size != texture.size.height {
            texture.resize(Extent3d {
                width: canvas.size,
                height: canvas.size,
                depth: 1,
            });
            material.grid_res = Vec2::new(canvas.size as f32, canvas.size as f32);
        }

        // Convert cells into bit-packed RGBA values and save them in the texture.
        let mut i = 0;
        for y in 0..canvas.size {
            for x in 0..canvas.size {
                canvas
                    .get_cell(IVec2::new(x as i32, y as i32))
                    .pack_into_4_bytes(&mut texture.data[i..(i + 4)]);

                i += 4;
            }
        }
    }
}
