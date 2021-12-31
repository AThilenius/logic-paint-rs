use std::collections::HashMap;
use std::mem::transmute;

use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::{
    substrate::{Cell, IntegratedCircuit, MosfetPart, Placement, SimIcState},
    unwrap_or_return,
    wgl2::{CellProgram, QuadVao, Texture, LOG_CHUNK_SIZE},
};

use super::{buffer::BufferChunk, CellCoord, ChunkCoord};

/// A positioned texture quad that draws a fixed-size "chunk" of an infinite Substrate.
pub struct RenderChunk {
    buffer_chunk_generation: usize,
    chunk_loc: ChunkCoord,
    chunk_start_cell: CellCoord,
    gl: WebGl2RenderingContext,
    texture: Texture,
    vao: QuadVao,
}

impl RenderChunk {
    /// Creates a new render chunk for the specific chunk location.
    pub fn new(
        gl: &WebGl2RenderingContext,
        program: &CellProgram,
        chunk_loc: &ChunkCoord,
    ) -> Result<RenderChunk, JsValue> {
        let texture = Texture::new_chunk_texture(gl)?;
        let vao = QuadVao::new(gl, program, chunk_loc)?;

        Ok(RenderChunk {
            buffer_chunk_generation: usize::MAX,
            gl: gl.clone(),
            chunk_loc: chunk_loc.clone(),
            chunk_start_cell: chunk_loc.first_cell_coord(),
            texture,
            vao,
        })
    }

    pub fn draw(&mut self, buffer_chunk: &BufferChunk) -> Result<(), JsValue> {
        if self.buffer_chunk_generation != buffer_chunk.generation {
            self.buffer_chunk_generation = buffer_chunk.generation;
            self.texture.set_pixels(&buffer_chunk.cells[..])?;
        }

        self.texture.bind();
        self.vao.bind();
        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        Ok(())
    }
}
