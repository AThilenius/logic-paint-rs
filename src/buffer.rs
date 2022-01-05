use std::{collections::HashMap, iter::FromIterator};

use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    coords::CHUNK_SIZE,
    module::Module,
    range::Range,
    upc::{LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
};

use super::coords::{CellCoord, ChunkCoord, LocalCoord, LOG_CHUNK_SIZE};

#[derive(Default, Serialize, Deserialize)]
pub struct Buffer {
    chunks: HashMap<ChunkCoord, BufferChunk>,
    transact_chunks: HashMap<ChunkCoord, BufferChunk>,
    transact: bool,
    undo_stack: Vec<BufferSnapshot>,
    redo_stack: Vec<BufferSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BufferChunk {
    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<u8>,

    /// How many cells are non-default.
    pub cell_count: usize,

    /// Modules, by module root-node coords.
    pub modules: HashMap<CellCoord, Module>,

    /// The generation number of this chunk. Monotonically increasing each mutation.
    pub generation: usize,
}

#[derive(Serialize, Deserialize)]
struct BufferSnapshot {
    pub chunks: HashMap<ChunkCoord, Option<BufferChunk>>,
}

impl Buffer {
    pub fn get_chunk<T>(&self, c: T) -> Option<&BufferChunk>
    where
        T: Into<ChunkCoord>,
    {
        let coord: ChunkCoord = c.into();
        self.transact_chunks
            .get(&coord)
            .or_else(|| self.chunks.get(&coord))
    }

    /// Returns an iterator for only committed chunk data (ignoring any pending transactions).
    pub fn get_base_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &BufferChunk)> {
        self.chunks.iter()
    }

    pub fn get_cell<T>(&self, c: T) -> UPC
    where
        T: Into<CellCoord>,
    {
        let coord: CellCoord = c.into();
        if let Some(chunk) = self.get_chunk(coord) {
            chunk.get_cell(coord)
        } else {
            Default::default()
        }
    }

    pub fn transaction_begin(&mut self) {
        debug_assert!(!self.transact);

        self.transact = true;
        self.transact_chunks.clear();
    }

    pub fn transaction_abort(&mut self) {
        debug_assert!(self.transact);

        self.transact = false;
        self.transact_chunks.clear();
    }

    pub fn transaction_commit(&mut self, push_undo_stack: bool) {
        debug_assert!(self.transact);

        if push_undo_stack {
            self.undo_stack
                .push(self.snapshot_chunks(self.transact_chunks.keys()));
            self.redo_stack.clear();
        }

        self.transact = false;
        for (coord, chunk) in self.transact_chunks.drain() {
            if chunk.cell_count == 0 && chunk.modules.len() == 0 {
                self.chunks.remove(&coord);
            } else {
                self.chunks.insert(coord, chunk);
            }
        }
    }

    pub fn transaction_undo(&mut self) {
        debug_assert!(!self.transact);

        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack
                .push(self.snapshot_chunks(snapshot.chunks.keys()));
            self.snapshot_apply(snapshot);
        }
    }

    pub fn transaction_redo(&mut self) {
        debug_assert!(!self.transact);

        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack
                .push(self.snapshot_chunks(snapshot.chunks.keys()));
            self.snapshot_apply(snapshot);
        }
    }

    pub fn transact_set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<CellCoord>,
    {
        debug_assert!(self.transact);

        let coord: CellCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        let chunk = if let Some(chunk) = self.transact_chunks.get_mut(&chunk_coord) {
            chunk
        } else {
            self.transact_chunks.insert(
                chunk_coord,
                self.get_chunk(chunk_coord).cloned().unwrap_or_default(),
            );
            self.transact_chunks.get_mut(&chunk_coord).unwrap()
        };

        chunk.generation += 1;
        chunk.set_cell(coord, cell);
    }

    pub fn transact_set_chunk<T>(&mut self, c: T, mut chunk: BufferChunk)
    where
        T: Into<ChunkCoord>,
    {
        debug_assert!(self.transact);

        let chunk_coord: ChunkCoord = c.into();
        let gen = self
            .get_chunk(chunk_coord)
            .map(|c| c.generation)
            .unwrap_or(0);

        chunk.generation = gen;
        self.transact_chunks.insert(chunk_coord, chunk);
    }

    pub fn clone_range(&self, range: Range) -> Buffer {
        // TODO: This can be made much more efficient (broken iter over effected chunks), but I
        // probably don't care. It's only executed in human-time. -shrug-
        let mut buf = Buffer::default();
        buf.transaction_begin();
        for cell_coord in range.iter_cell_coords() {
            buf.transact_set_cell(cell_coord, self.get_cell(cell_coord));
        }
        buf.transaction_commit(false);
        buf
    }

    pub fn extend(&mut self, data: &Buffer, offset: &IVec2) {
        todo!()
    }

    fn snapshot_chunks<'a, T>(&'a self, chunks: T) -> BufferSnapshot
    where
        T: Iterator<Item = &'a ChunkCoord>,
    {
        BufferSnapshot {
            chunks: HashMap::from_iter(
                chunks.map(|coord| (*coord, self.chunks.get(coord).cloned())),
            ),
        }
    }

    fn snapshot_apply(&mut self, snapshot: BufferSnapshot) {
        for (coord, chunk) in snapshot.chunks {
            if let Some(chunk) = chunk {
                self.chunks.insert(coord, chunk);
            } else {
                self.chunks.remove(&coord);
            }
        }
    }
}

impl BufferChunk {
    #[inline(always)]
    pub fn get_cell<T>(&self, c: T) -> UPC
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = ((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize;
        UPC::from_slice(&self.cells[idx..idx + UPC_BYTE_LEN])
    }

    #[inline(always)]
    pub fn set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
        let slice = &mut self.cells[idx..idx + UPC_BYTE_LEN];
        let existing = UPC::from_slice(slice);

        // Track cell count as well.
        if existing == Default::default() && cell != Default::default() {
            self.cell_count += 1;
        } else if existing != Default::default() && cell == Default::default() {
            self.cell_count -= 1;
        }

        slice.copy_from_slice(&cell.0);
    }
}

impl Default for BufferChunk {
    fn default() -> Self {
        Self {
            cells: vec![Default::default(); UPC_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE],
            cell_count: 0,
            modules: Default::default(),
            generation: 0,
        }
    }
}
