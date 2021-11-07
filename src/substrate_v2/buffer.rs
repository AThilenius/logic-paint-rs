use std::{collections::HashMap, iter::FromIterator};

use glam::IVec2;

use crate::substrate_v2::coords::CHUNK_SIZE;

use super::{
    coords::{CellCoord, ChunkCoord, LocalCoord, LOG_CHUNK_SIZE},
    module::Module,
    range::Range,
    upc::UPC,
};

/// Buffer: Unpacked Blueprint, in UPC format with an optional undo stack.
/// - Chunks (for cells) are blittable to the GPU and can be quickly serialized to Blueprints.
///   Modules are custom rendered, thus will never be blittable.
/// - Almost always wrapped in a Rc<RefCell<T>> for shared ownership.
/// - Supports beginning, canceling, and committing a mutation transaction:
///   - Start of a transaction conceptually marks the 'before state' for an undo Blueprint.
///   - Mutating during a transaction is done by cloning the effected Chunk, and mutating the clone
///     in-place.
///     - A chunk is considered mutated when any bit in the UPC cells changes, or a bit in the
///       serialized module data changes (again, does not include the contents of blob refs).
///   - Cancelling a transaction effectively throws away all cloned chunks / modules.
///   - Committing first takes a snapshot of mutated chunks and modules in their starting state (if
///     the undo stack is enabled) before replacing the base chunks with the mutated ones.
/// - Keeps a generation counter (usize) for each chunk, which allows rendering code to quickly
///   check for mutation. The counter is rolled back when a transaction is canceled.
/// - Supports undo by keeping a stack of Blueprints, each representing a subset of buffer chunks
///   and modules from BEFORE a change was made. Overwrites each chunk in the undo frame.
#[derive(Default)]
pub struct Buffer {
    chunks: HashMap<ChunkCoord, BufferChunk>,
    transact_chunks: HashMap<ChunkCoord, BufferChunk>,
    transact: bool,
    undo_stack: Vec<BufferSnapshot>,
    redo_stack: Vec<BufferSnapshot>,
}

#[derive(Clone)]
pub struct BufferChunk {
    /// Cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<UPC>,

    /// How many cells are non-default.
    pub cell_count: usize,

    /// Modules, by module root-node coords.
    pub modules: HashMap<CellCoord, Module>,
}

pub struct BufferSnapshot {
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

    pub fn get_cell<T>(&self, c: T) -> UPC
    where
        T: Into<CellCoord>,
    {
        let coord: CellCoord = c.into();
        if let Some(chunk) = self.get_chunk(coord) {
            chunk.get_cell(coord)
        } else {
            0
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
            self.transact_chunks.insert(chunk_coord, Default::default());
            self.transact_chunks.get_mut(&chunk_coord).unwrap()
        };

        chunk.set_cell(coord, cell);
    }

    pub fn clone_range(&self, range: Range) -> Buffer {
        match range {
            Range::Rectangle {
                lower_left,
                upper_right,
            } => {
                // TODO: This can be made much more efficient, but I might not care -shrug-
                let mut buf = Buffer::default();
                buf.transaction_begin();
                for y in lower_left.0.y..upper_right.0.y {
                    for x in lower_left.0.x..upper_right.0.x {
                        buf.transact_set_cell((x, y), self.get_cell((x, y)));
                    }
                }
                buf.transaction_commit(false);
                buf
            }
        }
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
        self.cells[((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize]
    }

    #[inline(always)]
    pub fn set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let slot = &mut self.cells[((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize];

        // Track cell count as well.
        if *slot == Default::default() && cell != Default::default() {
            self.cell_count += 1;
        } else if *slot != Default::default() && cell == Default::default() {
            self.cell_count -= 1;
        }

        *slot = cell;
    }
}

impl Default for BufferChunk {
    fn default() -> Self {
        Self {
            cells: vec![0u32; CHUNK_SIZE * CHUNK_SIZE],
            cell_count: 0,
            modules: Default::default(),
        }
    }
}
