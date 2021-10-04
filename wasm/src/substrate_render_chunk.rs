use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::wgl2::CellTexture;

pub const RENDER_CHUNK_SIZE: usize = 32;

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct SubstrateRenderChunk {
    cell_texture: CellTexture,
}

impl SubstrateRenderChunk {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<SubstrateRenderChunk, JsValue> {
        let cell_texture = CellTexture::new(&ctx)?;

        let mut pixels = vec![255u8; 4 * RENDER_CHUNK_SIZE * RENDER_CHUNK_SIZE];
        let mut i = 0;
        for y in 0..RENDER_CHUNK_SIZE {
            for x in 0..RENDER_CHUNK_SIZE {
                pixels[i + 0] = (x * 4) as u8;
                pixels[i + 1] = (y * 4) as u8;
                pixels[i + 2] = 0;
                pixels[i + 3] = 255;
                i += 4;
            }
        }
        cell_texture.set_pixels(ctx, &pixels)?;

        Ok(SubstrateRenderChunk { cell_texture })
    }
}
