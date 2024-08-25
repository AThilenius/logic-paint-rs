use wasm_bindgen::prelude::*;

use crate::{
    coords::{ChunkCoord, LocalCoord, CHUNK_SIZE},
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart, CompilerResults},
    },
};

/// Number of bytes used per cell in a BufferMask
pub const MASK_BYTE_LEN: usize = 4;

/// Much like a Buffer, except lacking any undo or transaction support. Designed to 'overlay' a
/// buffer, activating various atoms. Any active atom that does not overlay a cell is considered
/// undefined behavior.
#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct Mask {
    chunks: im::HashMap<ChunkCoord, BufferMaskChunk>,
}

#[allow(dead_code)]
impl Mask {
    pub fn from_highlight_trace(buffer: &Buffer, atom: Atom) -> Mask {
        let mut mask = Mask::default();
        let trace = CompilerResults::get_trace_atoms(buffer, atom);

        for atom in trace {
            let chunk_coord: ChunkCoord = atom.coord.into();
            let local_coord: LocalCoord = atom.coord.into();

            let chunk = mask.get_or_create_chunk_mut(chunk_coord);
            let i = local_coord.to_upc_idx();
            match atom.part {
                CellPart::Metal => chunk.cells[i + 0] = 1,
                CellPart::Si => chunk.cells[i + 1] = 1,
                CellPart::EcUpLeft => chunk.cells[i + 2] = 1,
                CellPart::EcDownRight => chunk.cells[i + 3] = 1,
            }
        }

        mask
    }

    pub fn get_chunk<T>(&self, c: T) -> Option<&BufferMaskChunk>
    where
        T: Into<ChunkCoord>,
    {
        let coord: ChunkCoord = c.into();
        self.chunks.get(&coord)
    }

    pub fn get_or_create_chunk<T>(&mut self, c: T) -> &BufferMaskChunk
    where
        T: Into<ChunkCoord>,
    {
        let coord: ChunkCoord = c.into();

        match self.chunks.entry(coord) {
            im::hashmap::Entry::Occupied(o) => o.into_mut(),
            im::hashmap::Entry::Vacant(v) => v.insert(Default::default()),
        }
    }

    pub fn get_or_create_chunk_mut<T>(&mut self, c: T) -> &mut BufferMaskChunk
    where
        T: Into<ChunkCoord>,
    {
        let coord: ChunkCoord = c.into();

        match self.chunks.entry(coord) {
            im::hashmap::Entry::Occupied(o) => o.into_mut(),
            im::hashmap::Entry::Vacant(v) => v.insert(Default::default()),
        }
    }
}

#[derive(Clone)]
pub struct BufferMaskChunk {
    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<u8>,
}

impl Default for BufferMaskChunk {
    fn default() -> Self {
        Self {
            cells: vec![Default::default(); MASK_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}
