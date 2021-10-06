use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::{
    log,
    substrate::{IntegratedCircuit, Metal, Silicon, CHUNK_SIZE, LOG_CHUNK_SIZE},
    wgl2::CellTexture,
};

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct SubstrateRenderChunk {
    cell_texture: CellTexture,
}

impl SubstrateRenderChunk {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<SubstrateRenderChunk, JsValue> {
        Ok(SubstrateRenderChunk {
            cell_texture: CellTexture::new(&ctx)?,
        })
    }

    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        self.cell_texture.bind(&ctx);
    }

    pub fn rasterize_ic_chunk(
        &mut self,
        ctx: &WebGl2RenderingContext,
        ic: &IntegratedCircuit,
        chunk_loc: &IVec2,
    ) -> Result<(), JsValue> {
        let chunk = if let Some(cells) = ic.get_chunk(&chunk_loc) {
            cells
        } else {
            return Ok(());
        };

        let start = IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);
        let mut pixels = vec![0u8; 4 * CHUNK_SIZE * CHUNK_SIZE];
        let mut i = 0;
        for y in start.y..(start.y + CHUNK_SIZE as i32) {
            for x in start.x..(start.x + CHUNK_SIZE as i32) {
                if let Some(cell) = chunk.get(&IVec2::new(x, y)) {
                    let buf = &mut pixels[i..];

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

                i += 4;
            }
        }

        self.cell_texture.set_pixels(&ctx, &pixels)
    }
}
