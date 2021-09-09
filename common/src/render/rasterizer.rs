use bevy::{prelude::*, render::texture::Extent3d};
use packed_struct::prelude::*;

use crate::{
    canvas::{CanvasCell, CanvasData, MetalLayer},
    render::CellMaterial,
};

#[derive(PrimitiveEnum, Debug, Copy, Clone, PartialEq)]
pub enum SiLayerType {
    None = 0b00,
    PType = 0b01,
    NType = 0b10,
}

#[derive(PrimitiveEnum, Debug, Copy, Clone, PartialEq)]
pub enum LayerState {
    Off = 0b00,
    On = 0b01,
    Err = 0b10,
}

impl From<crate::canvas::LayerState> for LayerState {
    #[inline(always)]
    fn from(s: crate::canvas::LayerState) -> Self {
        match s {
            crate::canvas::LayerState::Off => LayerState::Off,
            crate::canvas::LayerState::On => LayerState::On,
            crate::canvas::LayerState::Err => LayerState::Err,
        }
    }
}

#[derive(PackedStruct, Debug, Copy, Clone, PartialEq)]
#[packed_struct(bit_numbering = "msb0")]
pub struct PackedPixel {
    // First 8 bits
    #[packed_field(bits = "0:1", ty = "enum")]
    lower_si_layer: SiLayerType,
    #[packed_field(bits = "2:3", ty = "enum")]
    upper_si_layer: SiLayerType,
    #[packed_field(bits = "4:5", ty = "enum")]
    si_lower_state: LayerState,
    #[packed_field(bits = "6:7", ty = "enum")]
    si_upper_state: LayerState,

    // Second 8 bits
    has_metal_layer: bool,
    has_via: bool,
    #[packed_field(bits = "10:11", ty = "enum")]
    metal_layer_state: LayerState,
}

impl From<&CanvasCell> for PackedPixel {
    #[inline(always)]
    fn from(c: &CanvasCell) -> Self {
        Self {
            lower_si_layer: match c.si {
                crate::SiLayer::N => SiLayerType::NType,
                crate::SiLayer::P => SiLayerType::PType,
                crate::SiLayer::POnN => SiLayerType::NType,
                crate::SiLayer::NOnP => SiLayerType::PType,
                _ => SiLayerType::None,
            },
            upper_si_layer: match c.si {
                crate::SiLayer::POnN => SiLayerType::PType,
                crate::SiLayer::NOnP => SiLayerType::NType,
                _ => SiLayerType::None,
            },
            si_lower_state: c.si_lower_state.into(),
            si_upper_state: c.si_upper_state.into(),
            has_metal_layer: match c.metal {
                MetalLayer::Metal | MetalLayer::MetalAndVia => true,
                _ => false,
            },
            has_via: c.metal == MetalLayer::MetalAndVia,
            metal_layer_state: c.metal_state.into(),
        }
    }
}

pub fn render_canvas_to_texture(
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<CellMaterial>>,
    query: Query<(&CanvasData, &Handle<CellMaterial>)>,
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
