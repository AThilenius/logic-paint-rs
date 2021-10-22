use std::collections::HashMap;

use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    substrate::{Cell, IntegratedCircuit, MosfetPart, Placement, SimIcState},
    unwrap_or_return,
    wgl2::{cell_to_chunk_loc, LOG_CHUNK_SIZE},
};

use super::CHUNK_SIZE;

// Red component bit masks.
const _FLAG_R_UPPER_LEFT_SI_ACTIVE: u8 = 1u8 << 7;
const FLAG_R_SI_N: u8 = 1u8 << 6;
const FLAG_R_SI_P: u8 = 1u8 << 5;
const FLAG_R_SI_DIR_UP: u8 = 1u8 << 4;
const FLAG_R_SI_DIR_RIGHT: u8 = 1u8 << 3;
const FLAG_R_SI_DIR_DOWN: u8 = 1u8 << 2;
const FLAG_R_SI_DIR_LEFT: u8 = 1u8 << 1;

// Green component bit masks.
const _FLAG_G_LOWER_RIGHT_SI_ACTIVE: u8 = 1u8 << 7;
const FLAG_G_GATE_DIR_UP: u8 = 1u8 << 6;
const FLAG_G_GATE_DIR_RIGHT: u8 = 1u8 << 5;
const FLAG_G_GATE_DIR_DOWN: u8 = 1u8 << 4;
const FLAG_G_GATE_DIR_LEFT: u8 = 1u8 << 3;

// Blue component bit masks.
const _FLAG_B_GATE_SI_ACTIVE: u8 = 1u8 << 7;
const FLAG_B_METAL: u8 = 1u8 << 6;
const FLAG_B_METAL_DIR_UP: u8 = 1u8 << 5;
const FLAG_B_METAL_DIR_RIGHT: u8 = 1u8 << 4;
const FLAG_B_METAL_DIR_DOWN: u8 = 1u8 << 3;
const FLAG_B_METAL_DIR_LEFT: u8 = 1u8 << 2;
const FLAG_B_VIA: u8 = 1u8 << 1;

// Alpha component bit masks.
const _FLAG_A_METAL_ACTIVE: u8 = 1u8 << 7;

// The *_ACTIVE flags are all the same as far as code cares. They only exist above for organization.
const FLAG_ACTIVE: u8 = 1u8 << 7;
const SI_UPPER_LEFT_ACTIVE_COMPONENT_OFFSET: usize = 0;
const SI_LOWER_RIGHT_ACTIVE_COMPONENT_OFFSET: usize = 1;
const SI_BASE_ACTIVE_COMPONENT_OFFSET: usize = 2;
const METAL_ACTIVE_COMPONENT_OFFSET: usize = 3;

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct CellRenderChunk {
    blank: bool,
    texture: WebGlTexture,
    pixels: Vec<u8>,
    texel_component_to_trace_handle: Vec<(usize, usize)>,
}

impl CellRenderChunk {
    /// Creates a new render chunk without an affinity to a specific location.
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<CellRenderChunk, JsValue> {
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
            0,
            WebGl2RenderingContext::RGBA8UI as i32,
            CHUNK_SIZE as i32,
            CHUNK_SIZE as i32,
            0,
            WebGl2RenderingContext::RGBA_INTEGER,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        )?;

        Ok(CellRenderChunk {
            texture,
            blank: true,
            pixels: vec![0u8; 4 * CHUNK_SIZE * CHUNK_SIZE],
            texel_component_to_trace_handle: Vec::new(),
        })
    }

    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn update_trace_active(
        &mut self,
        ctx: &WebGl2RenderingContext,
        sim_state: &SimIcState,
    ) -> Result<(), JsValue> {
        for (texel_cmp, trace_handle) in &self.texel_component_to_trace_handle {
            let active = sim_state.trace_states[*trace_handle];
            if active {
                self.pixels[*texel_cmp] |= FLAG_ACTIVE;
            } else {
                self.pixels[*texel_cmp] ^= FLAG_ACTIVE;
            }
        }

        self.upload_pixels(ctx)
    }

    /// Resets all data back to the initial empty state and rebuilds just the layout texture. Does
    /// not rebuild the trace state chunk however.
    pub fn reset_and_update_layout(
        &mut self,
        ctx: &WebGl2RenderingContext,
        ic: &IntegratedCircuit,
        overrides: &HashMap<IVec2, Cell>,
        chunk_loc: &IVec2,
    ) -> Result<(), JsValue> {
        self.texel_component_to_trace_handle.clear();
        self.pixels.iter_mut().for_each(|m| *m = 0);

        let start_cell = IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);

        // Collect all cells that will be drawn in this chunk.
        let override_cells = overrides
            .iter()
            .filter(|(loc, _)| cell_to_chunk_loc(loc) == *chunk_loc);
        let cells: Vec<_> = if let Some(chunk) = ic.get_cell_chunk_by_chunk_location(&chunk_loc) {
            chunk.iter().chain(override_cells).collect()
        } else {
            override_cells.collect()
        };

        // Short-circuit empty drawing
        if cells.len() == 0 && self.blank {
            return Ok(());
        }
        self.blank = cells.len() == 0;

        for (loc, cell) in cells {
            // To chunk-local coords, then: ((loc.y * CHUNK_SIZE) + loc.x) * 4
            let loc = *loc - start_cell;
            let i = ((loc.y << LOG_CHUNK_SIZE) + loc.x) << 2;
            let pixel = &mut self.pixels[i as usize..];

            for atom in cell {
                // Metal can only be on one atom
                if atom.metal != Placement::NONE {
                    pixel[2] |= FLAG_B_METAL;

                    if atom.metal.up {
                        pixel[2] |= FLAG_B_METAL_DIR_UP;
                    };
                    if atom.metal.right {
                        pixel[2] |= FLAG_B_METAL_DIR_RIGHT;
                    };
                    if atom.metal.down {
                        pixel[2] |= FLAG_B_METAL_DIR_DOWN;
                    };
                    if atom.metal.left {
                        pixel[2] |= FLAG_B_METAL_DIR_LEFT;
                    };
                    if atom.si != Placement::NONE {
                        pixel[2] |= FLAG_B_VIA;
                    }
                }

                if atom.si != Placement::NONE {
                    match atom.mosfet_part {
                        MosfetPart::None | MosfetPart::LeftEC | MosfetPart::RightEC => {
                            if atom.si.up {
                                pixel[0] |= FLAG_R_SI_DIR_UP;
                            }
                            if atom.si.right {
                                pixel[0] |= FLAG_R_SI_DIR_RIGHT;
                            }
                            if atom.si.down {
                                pixel[0] |= FLAG_R_SI_DIR_DOWN;
                            }
                            if atom.si.left {
                                pixel[0] |= FLAG_R_SI_DIR_LEFT;
                            }
                            if atom.is_si_n {
                                pixel[0] |= FLAG_R_SI_N;
                            } else {
                                // Note: N and P are separate to denote a cell with Si but no
                                // connections (are dirs are false but it still has Si).
                                pixel[0] |= FLAG_R_SI_P;
                            }
                        }
                        MosfetPart::Base => {
                            if atom.si.up {
                                pixel[1] |= FLAG_G_GATE_DIR_UP;
                            }
                            if atom.si.right {
                                pixel[1] |= FLAG_G_GATE_DIR_RIGHT;
                            }
                            if atom.si.down {
                                pixel[1] |= FLAG_G_GATE_DIR_DOWN;
                            }
                            if atom.si.left {
                                pixel[1] |= FLAG_G_GATE_DIR_LEFT;
                            }
                        }
                    }
                }
            }
        }

        self.upload_pixels(ctx)
    }

    /// Rebuilds the trace state look-aside cache. This should be called after IC mutation is
    /// complete.
    pub fn rebuild_trace_state_lookaside_cache(
        &mut self,
        ic: &IntegratedCircuit,
        chunk_loc: &IVec2,
    ) {
        let start_cell = IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);

        // Nothing interesting to be done if the chunk is empty, so unwrap or return.
        let cells = unwrap_or_return!(ic
            .get_cell_chunk_by_chunk_location(&chunk_loc)
            .map(|chunk| chunk.iter()));

        for (loc, cell) in cells {
            // To chunk-local coords, then: ((loc.y * CHUNK_SIZE) + loc.x) * 4
            let loc = *loc - start_cell;
            let i = (((loc.y << LOG_CHUNK_SIZE) + loc.x) << 2) as usize;

            for atom in cell {
                if let Some(trace_handle) = ic.get_trace_handle_by_atom(atom) {
                    // Associate the atom type with the trace handle
                    if atom.metal != Placement::NONE {
                        self.texel_component_to_trace_handle
                            .push((i + METAL_ACTIVE_COMPONENT_OFFSET, trace_handle));
                    }

                    if atom.si != Placement::NONE {
                        match atom.mosfet_part {
                            MosfetPart::None => {
                                // It's all the same, normal Si trace.
                                self.texel_component_to_trace_handle.push((
                                    i + SI_UPPER_LEFT_ACTIVE_COMPONENT_OFFSET,
                                    trace_handle,
                                ));
                                self.texel_component_to_trace_handle.push((
                                    i + SI_LOWER_RIGHT_ACTIVE_COMPONENT_OFFSET,
                                    trace_handle,
                                ));
                            }
                            MosfetPart::Base => {
                                self.texel_component_to_trace_handle
                                    .push((i + SI_BASE_ACTIVE_COMPONENT_OFFSET, trace_handle));
                            }
                            MosfetPart::LeftEC => {
                                self.texel_component_to_trace_handle.push((
                                    i + SI_UPPER_LEFT_ACTIVE_COMPONENT_OFFSET,
                                    trace_handle,
                                ));
                            }
                            MosfetPart::RightEC => {
                                self.texel_component_to_trace_handle.push((
                                    i + SI_LOWER_RIGHT_ACTIVE_COMPONENT_OFFSET,
                                    trace_handle,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Then sort the buffer by texel component index for cache-coherency.
        self.texel_component_to_trace_handle
            .sort_by_key(|(texel_cmp, _)| *texel_cmp);
    }

    fn upload_pixels(&self, ctx: &WebGl2RenderingContext) -> Result<(), JsValue> {
        self.bind(&ctx);
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA8UI as i32,
            CHUNK_SIZE as i32,
            CHUNK_SIZE as i32,
            0,
            WebGl2RenderingContext::RGBA_INTEGER,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&self.pixels[..]),
        )
    }
}
