use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

const RENDER_CHUNK_SIZE: usize = 32;

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct SubstrateRenderChunk {
    gl_texture: Rc<WebGlTexture>,
}

impl SubstrateRenderChunk {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<SubstrateRenderChunk, JsValue> {
        let texture = ctx.create_texture().expect("Cannot create gl texture");
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // Integer texture types require NEAREST filtering. Also clamp to texture edges.
        ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        let pixel: &[u8] = &[128u8; 4 * RENDER_CHUNK_SIZE * RENDER_CHUNK_SIZE];
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,                                      // Level
            WebGl2RenderingContext::RGBA8UI as i32, // Internal format
            RENDER_CHUNK_SIZE as i32,
            RENDER_CHUNK_SIZE as i32,
            0,                                     // Border
            WebGl2RenderingContext::RGBA_INTEGER,  // Src format
            WebGl2RenderingContext::UNSIGNED_BYTE, // Src type
            Some(&pixel),
        )?;

        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        Ok(SubstrateRenderChunk {
            gl_texture: Rc::new(texture),
        })
    }
}
