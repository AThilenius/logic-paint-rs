use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    log,
    substrate::{
        cell_to_chunk_loc, Cell, IntegratedCircuit, Metal, Silicon, CHUNK_SIZE, LOG_CHUNK_SIZE,
    },
};

thread_local! {
    pub static PIXEL_BUF: RefCell<[u8; 4 * CHUNK_SIZE * CHUNK_SIZE]> =
        RefCell::new([0_u8; 4 * CHUNK_SIZE * CHUNK_SIZE]);
}

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct CellChunkTexture {
    blank: bool,
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

        Ok(CellChunkTexture {
            texture,
            blank: true,
        })
    }

    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn rasterize_ic_chunk(
        &mut self,
        ctx: &WebGl2RenderingContext,
        ic: &IntegratedCircuit,
        overrides: &HashMap<IVec2, Cell>,
        chunk_loc: &IVec2,
    ) -> Result<(), JsValue> {
        let start = IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);

        // Collect all cells that will be drawn in this chunk.
        let override_cells = overrides
            .iter()
            .filter(|(loc, _)| cell_to_chunk_loc(loc) == *chunk_loc);
        let cells: Vec<_> = if let Some(chunk) = ic.get_chunk(&chunk_loc) {
            chunk.iter().chain(override_cells).collect()
        } else {
            override_cells.collect()
        };

        // Short-circuit empty drawing
        if cells.len() == 0 && self.blank {
            return Ok(());
        }
        self.blank = cells.len() == 0;

        PIXEL_BUF.with(|pixels| {
            let mut pixels = pixels.borrow_mut();
            for (loc, cell) in cells {
                // To chunk-local coords, then: ((loc.y * CHUNK_SIZE) + loc.x) * 4
                let loc = *loc - start;
                let i = ((loc.y << LOG_CHUNK_SIZE) + loc.x) << 2;
                let buf = &mut pixels[i as usize..];

                // Bit field masks (3 bytes)
                let si_n = 1u8 << 7;
                let si_p = 1u8 << 6;
                let si_active = 1u8 << 5;
                let si_dir_up = 1u8 << 4;
                let si_dir_right = 1u8 << 3;
                let si_dir_down = 1u8 << 2;
                let si_dir_left = 1u8 << 1;

                let gate_dir_up = 1u8 << 7;
                let gate_dir_right = 1u8 << 6;
                let gate_dir_down = 1u8 << 5;
                let gate_dir_left = 1u8 << 4;
                let gate_active = 1u8 << 3;

                let metal = 1u8 << 7;
                let metal_dir_up = 1u8 << 6;
                let metal_dir_right = 1u8 << 5;
                let metal_dir_down = 1u8 << 4;
                let metal_dir_left = 1u8 << 3;
                let metal_active = 1u8 << 2;
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
                    Silicon::NP { path, .. } => {
                        buf[0] |= if ic.get_path_dc(path) > 0 {
                            si_active
                        } else {
                            0
                        };
                    }
                    Silicon::Mosfet {
                        gate_dirs, path, ..
                    } => {
                        buf[1] |= if gate_dirs.up { gate_dir_up } else { 0 };
                        buf[1] |= if gate_dirs.right { gate_dir_right } else { 0 };
                        buf[1] |= if gate_dirs.down { gate_dir_down } else { 0 };
                        buf[1] |= if gate_dirs.left { gate_dir_left } else { 0 };

                        buf[1] |= if ic.get_path_dc(path) > 0 {
                            gate_active
                        } else {
                            0
                        };
                    }
                    _ => {}
                }

                match cell.metal {
                    Metal::IO { dirs, path, .. } | Metal::Trace { dirs, path, .. } => {
                        buf[2] |= metal;
                        buf[2] |= if dirs.up { metal_dir_up } else { 0 };
                        buf[2] |= if dirs.right { metal_dir_right } else { 0 };
                        buf[2] |= if dirs.down { metal_dir_down } else { 0 };
                        buf[2] |= if dirs.left { metal_dir_left } else { 0 };

                        buf[2] |= if ic.get_path_dc(path) > 0 {
                            metal_active
                        } else {
                            0
                        };
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

            self.bind(&ctx);
            let ret = ctx
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,                                      // Level
                    WebGl2RenderingContext::RGBA8UI as i32, // Internal format
                    CHUNK_SIZE as i32,
                    CHUNK_SIZE as i32,
                    0,                                     // Border
                    WebGl2RenderingContext::RGBA_INTEGER,  // Src format
                    WebGl2RenderingContext::UNSIGNED_BYTE, // Src type
                    Some(&pixels[..]),
                );

            // Reset pixels array to zeros when done. Rust is magic and this turns into a single
            // memset call. So. Fucking. Cool. <3 Rust.
            pixels.borrow_mut().iter_mut().for_each(|m| *m = 0);

            ret
        })
    }
}
