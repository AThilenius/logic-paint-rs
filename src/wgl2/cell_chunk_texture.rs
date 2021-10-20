use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    substrate::{Cell, IntegratedCircuit, Metal, NormalizedCell, Silicon},
    wgl2::{cell_to_chunk_loc, LOG_CHUNK_SIZE},
};

use super::CHUNK_SIZE;

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

        // Collect all cell from the IC that belong to this chunk.
        let mut cells = vec![];
        for y in start.y..(start.y + CHUNK_SIZE as i32) {
            for x in start.x..(start.x + CHUNK_SIZE as i32) {
                if let Some(cell) = ic.get_cell_by_location(IVec2::new(x, y)) {
                    cells.push((IVec2::new(x, y), cell));
                }
            }
        }

        // Collect all cells that will be drawn in this chunk.
        cells.extend(
            overrides
                .iter()
                .filter(|(loc, _)| cell_to_chunk_loc(loc) == *chunk_loc)
                .map(|(l, c)| (*l, c.clone())),
        );

        // Short-circuit empty drawing
        if cells.len() == 0 && self.blank {
            return Ok(());
        }
        self.blank = cells.len() == 0;

        PIXEL_BUF.with(|pixels| {
            let mut pixels = pixels.borrow_mut();
            for (loc, cell) in cells {
                // TODO: Work directly with standard cells.
                let cell = NormalizedCell::from(cell.clone());

                // To chunk-local coords, then: ((loc.y * CHUNK_SIZE) + loc.x) * 4
                let loc = loc - start;
                let i = ((loc.y << LOG_CHUNK_SIZE) + loc.x) << 2;
                let buf = &mut pixels[i as usize..];

                // Bit field masks (3 bytes)
                let si_n = 1u8 << 7;
                let si_p = 1u8 << 6;
                let _si_active = 1u8 << 5;
                let si_pl_up = 1u8 << 4;
                let si_pl_right = 1u8 << 3;
                let si_pl_down = 1u8 << 2;
                let si_pl_left = 1u8 << 1;

                let gate_pl_up = 1u8 << 7;
                let gate_pl_right = 1u8 << 6;
                let gate_pl_down = 1u8 << 5;
                let gate_pl_left = 1u8 << 4;
                let _gate_active = 1u8 << 3;

                let metal = 1u8 << 7;
                let metal_pl_up = 1u8 << 6;
                let metal_pl_right = 1u8 << 5;
                let metal_pl_down = 1u8 << 4;
                let metal_pl_left = 1u8 << 3;
                let _metal_active = 1u8 << 2;
                let via = 1u8 << 1;
                let _is_io = 1u8 << 0;

                match cell.si {
                    Silicon::NP {
                        is_n, placement, ..
                    }
                    | Silicon::Mosfet {
                        is_npn: is_n,
                        ec_placement: placement,
                        ..
                    } => {
                        buf[0] |= if is_n { si_n } else { si_p };
                        buf[0] |= if placement.up { si_pl_up } else { 0 };
                        buf[0] |= if placement.right { si_pl_right } else { 0 };
                        buf[0] |= if placement.down { si_pl_down } else { 0 };
                        buf[0] |= if placement.left { si_pl_left } else { 0 };
                    }
                    _ => {}
                }

                match cell.si {
                    Silicon::NP { .. } => {
                        // TODO: Check active
                        // buf[0] |= if ic.get_path_dc(path) > 0 {
                        //     si_active
                        // } else {
                        //     0
                        // };
                    }
                    Silicon::Mosfet { gate_placement, .. } => {
                        buf[1] |= if gate_placement.up { gate_pl_up } else { 0 };
                        buf[1] |= if gate_placement.right {
                            gate_pl_right
                        } else {
                            0
                        };
                        buf[1] |= if gate_placement.down { gate_pl_down } else { 0 };
                        buf[1] |= if gate_placement.left { gate_pl_left } else { 0 };

                        // TODO: Check active
                        // buf[1] |= if ic.get_path_dc(path) > 0 {
                        //     gate_active
                        // } else {
                        //     0
                        // };
                    }
                    _ => {}
                }

                match cell.metal {
                    Metal::Trace { placement, .. } => {
                        buf[2] |= metal;
                        buf[2] |= if placement.up { metal_pl_up } else { 0 };
                        buf[2] |= if placement.right { metal_pl_right } else { 0 };
                        buf[2] |= if placement.down { metal_pl_down } else { 0 };
                        buf[2] |= if placement.left { metal_pl_left } else { 0 };

                        // TODO: Check active
                        // buf[2] |= if ic.get_path_dc(path) > 0 {
                        //     metal_active
                        // } else {
                        //     0
                        // };
                    }
                    Metal::None => {}
                }

                match cell.metal {
                    // TODO: Check for I/O at cell location.
                    // Metal::IO { .. } => {
                    //     buf[2] |= is_io;
                    // }
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
