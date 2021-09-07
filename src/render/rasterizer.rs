use bevy::{prelude::*, render::texture::Extent3d};
use packed_struct::prelude::*;

use crate::{
    canvas::{Canvas, CanvasCell},
    render::CellMaterial,
};

#[derive(PrimitiveEnum, Debug, Copy, Clone, PartialEq)]
pub enum SiLayerType {
    None = 0b00,
    PType = 0b01,
    NType = 0b10,
}

impl From<crate::canvas::SiLayerType> for SiLayerType {
    #[inline(always)]
    fn from(t: crate::canvas::SiLayerType) -> Self {
        match t {
            crate::canvas::SiLayerType::None => SiLayerType::None,
            crate::canvas::SiLayerType::PType => SiLayerType::PType,
            crate::canvas::SiLayerType::NType => SiLayerType::NType,
        }
    }
}

#[derive(PrimitiveEnum, Debug, Copy, Clone, PartialEq)]
pub enum LayerState {
    Inactive = 0b00,
    Active = 0b01,
    Error = 0b10,
}

impl From<crate::canvas::LayerState> for LayerState {
    #[inline(always)]
    fn from(s: crate::canvas::LayerState) -> Self {
        match s {
            crate::canvas::LayerState::Inactive => LayerState::Inactive,
            crate::canvas::LayerState::Active => LayerState::Active,
            crate::canvas::LayerState::Error => LayerState::Error,
        }
    }
}

#[derive(PackedStruct, Debug, Copy, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0")]
pub struct PackedPixel {
    #[packed_field(bits = "0:1", ty = "enum")]
    lower_si_layer: SiLayerType,
    #[packed_field(bits = "2:3", ty = "enum")]
    upper_si_layer: SiLayerType,
    #[packed_field(bits = "4:5", ty = "enum")]
    lower_si_layer_state: LayerState,
    #[packed_field(bits = "6:7", ty = "enum")]
    upper_si_layer_state: LayerState,
    has_metal_layer: bool,
    has_via: bool,
    #[packed_field(bits = "10:11", ty = "enum")]
    metal_layer_state: LayerState,
}

impl From<&CanvasCell> for PackedPixel {
    #[inline(always)]
    fn from(c: &CanvasCell) -> Self {
        Self {
            lower_si_layer: c.lower_si_layer.layer_type.into(),
            upper_si_layer: c.upper_si_layer.layer_type.into(),
            lower_si_layer_state: c.lower_si_layer.state.into(),
            upper_si_layer_state: c.upper_si_layer.state.into(),
            has_metal_layer: c.has_metal,
            has_via: c.has_via,
            metal_layer_state: c.metal_state.into(),
        }
    }
}

pub fn render_canvas_to_texture(
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<CellMaterial>>,
    query: Query<(&Canvas, &Handle<CellMaterial>)>,
) {
    for (canvas, cell_material_handle) in query.iter() {
        let material = materials.get_mut(cell_material_handle).unwrap();
        let texture = textures.get_mut(material.texture.clone()).unwrap();

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

        // Convert canval cells into packed pixels and save them in the texture.
        let mut i = 0;
        for y in 0..canvas.size {
            for x in 0..canvas.size {
                let pixel: PackedPixel = canvas.get_cell(IVec2::new(x as i32, y as i32)).into();
                let packed: [u8; 2] =
                    PackedPixel::pack(&pixel).expect("failed to pack texture data");

                texture.data[i] = packed[0];
                texture.data[i + 1] = packed[1];

                i += 4;
            }
        }
    }
}
