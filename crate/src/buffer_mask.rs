use std::collections::{hash_map::Entry, HashMap};

use crate::coords::{ChunkCoord, LocalCoord, CHUNK_SIZE, LOG_CHUNK_SIZE};

/// Number of bytes used per cell in a BufferMask
pub const MASK_BYTE_LEN: usize = 4;

/// Much like a Buffer, except lacking any undo or transaction support. Designed to 'overlay' a
/// buffer, activating various atoms. Any active atom that does not overlay a cell is considered
/// undefined behavior.
#[derive(Default)]
pub struct BufferMask {
    chunks: HashMap<ChunkCoord, BufferMaskChunk>,
}

#[allow(dead_code)]
impl BufferMask {
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
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Default::default()),
        }
    }

    pub fn get_or_create_chunk_mut<T>(&mut self, c: T) -> &mut BufferMaskChunk
    where
        T: Into<ChunkCoord>,
    {
        let coord: ChunkCoord = c.into();

        match self.chunks.entry(coord) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Default::default()),
        }
    }
}

pub struct BufferMaskChunk {
    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<u8>,
}

impl BufferMaskChunk {
    #[inline(always)]
    pub fn set_cell_active<T>(&mut self, c: T)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) * MASK_BYTE_LEN;
        self.cells[idx] = self.cells[idx] | (1u8 << 1);
    }
}

impl Default for BufferMaskChunk {
    fn default() -> Self {
        Self {
            cells: vec![Default::default(); MASK_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}
