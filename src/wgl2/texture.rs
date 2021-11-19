use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::v2::CHUNK_SIZE;

pub struct Texture {
    texture: WebGlTexture,
    gl: WebGl2RenderingContext,
}

impl Texture {
    pub fn new_chunk_texture(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let texture = gl.create_texture().expect("Cannot create gl texture");
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // Integer texture types require NEAREST filtering. Also clamp to texture edges.
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA8UI as i32,
            CHUNK_SIZE as i32,
            CHUNK_SIZE as i32,
            0,
            WebGl2RenderingContext::RGBA_INTEGER,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        )?;

        Ok(Self {
            texture,
            gl: gl.clone(),
        })
    }

    pub fn bind(&self) {
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn set_pixels(&self, pixels: &[u8]) -> Result<(), JsValue> {
        self.bind();
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA8UI as i32,
                CHUNK_SIZE as i32,
                CHUNK_SIZE as i32,
                0,
                WebGl2RenderingContext::RGBA_INTEGER,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(pixels),
            )
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.gl.delete_texture(Some(&self.texture));
    }
}
