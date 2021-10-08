use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::substrate::{IntegratedCircuit, Metal, Silicon, CHUNK_SIZE, LOG_CHUNK_SIZE};

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct CellChunkTexture {
    texture: WebGlTexture,
}

impl CellChunkTexture {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<CellChunkTexture, JsValue> {
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

        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,                                      // Level
            WebGl2RenderingContext::RGBA8UI as i32, // Internal format
            CHUNK_SIZE as i32,
            CHUNK_SIZE as i32,
            0,                                     // Border
            WebGl2RenderingContext::RGBA_INTEGER,  // Src format
            WebGl2RenderingContext::UNSIGNED_BYTE, // Src type
            None,
        )?;

        Ok(CellChunkTexture { texture })
    }

    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn rasterize_ic_chunk(
        &mut self,
        ctx: &WebGl2RenderingContext,
        ic: &IntegratedCircuit,
        chunk_loc: &IVec2,
    ) -> Result<(), JsValue> {
        let start = IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);
        let mut pixels = vec![0u8; 4 * CHUNK_SIZE * CHUNK_SIZE];
        if let Some(chunk) = ic.get_chunk(&chunk_loc) {
            for (loc, cell) in chunk {
                // To chunk-local coords, then: ((loc.y * CHUNK_SIZE) + loc.x) * 4
                let loc = *loc - start;
                let i = ((loc.y << LOG_CHUNK_SIZE) + loc.x) << 2;
                let buf = &mut pixels[i as usize..];

                // Bit field masks (3 bytes)
                let si_n = 1u8 << 7;
                let si_p = 1u8 << 6;
                // let si_active = 1u8 << 5;
                let si_dir_up = 1u8 << 4;
                let si_dir_right = 1u8 << 3;
                let si_dir_down = 1u8 << 2;
                let si_dir_left = 1u8 << 1;

                let gate_dir_up = 1u8 << 7;
                let gate_dir_right = 1u8 << 6;
                let gate_dir_down = 1u8 << 5;
                let gate_dir_left = 1u8 << 4;
                // let gate_active = 1u8 << 3;

                let metal = 1u8 << 7;
                let metal_dir_up = 1u8 << 6;
                let metal_dir_right = 1u8 << 5;
                let metal_dir_down = 1u8 << 4;
                let metal_dir_left = 1u8 << 3;
                // let metal_active = 1u8 << 2;
                let via = 1u8 << 1;
                let is_io = 1u8 << 0;

                match cell.si {
                    Silicon::NP { is_n, dirs, .. }
                    | Silicon::Mosfet {
                        is_npn: is_n,
                        ec_dirs: dirs,
                        ..
                    } => {
                        buf[0] |= if is_n { si_n } else { si_p };
                        buf[0] |= if dirs.up { si_dir_up } else { 0 };
                        buf[0] |= if dirs.right { si_dir_right } else { 0 };
                        buf[0] |= if dirs.down { si_dir_down } else { 0 };
                        buf[0] |= if dirs.left { si_dir_left } else { 0 };
                    }
                    _ => {}
                }

                match cell.si {
                    // Silicon::NP { is_n, .. } => {
                    //     buf[0] |= if is_n { si_n } else { si_p };
                    //     // TODO: Si active (1 << 5)
                    // }
                    Silicon::Mosfet { gate_dirs, .. } => {
                        buf[1] |= if gate_dirs.up { gate_dir_up } else { 0 };
                        buf[1] |= if gate_dirs.right { gate_dir_right } else { 0 };
                        buf[1] |= if gate_dirs.down { gate_dir_down } else { 0 };
                        buf[1] |= if gate_dirs.left { gate_dir_left } else { 0 };

                        // TODO: Gate/EC active
                    }
                    _ => {}
                }

                match cell.metal {
                    Metal::IO { dirs } | Metal::Trace { dirs, .. } => {
                        buf[2] |= metal;
                        buf[2] |= if dirs.up { metal_dir_up } else { 0 };
                        buf[2] |= if dirs.right { metal_dir_right } else { 0 };
                        buf[2] |= if dirs.down { metal_dir_down } else { 0 };
                        buf[2] |= if dirs.left { metal_dir_left } else { 0 };

                        // TODO: Metal active
                    }
                    Metal::None => {}
                }

                match cell.metal {
                    Metal::IO { .. } => {
                        buf[2] |= is_io;
                    }
                    Metal::Trace { has_via: true, .. } => {
                        buf[2] |= via;
                    }
                    _ => {}
                }
            }
        }

        self.bind(&ctx);
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,                                      // Level
            WebGl2RenderingContext::RGBA8UI as i32, // Internal format
            CHUNK_SIZE as i32,
            CHUNK_SIZE as i32,
            0,                                     // Border
            WebGl2RenderingContext::RGBA_INTEGER,  // Src format
            WebGl2RenderingContext::UNSIGNED_BYTE, // Src type
            Some(&pixels),
        )
    }
}
