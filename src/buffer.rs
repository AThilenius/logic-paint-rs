use std::{cell::RefCell, collections::HashMap, iter::FromIterator, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    coords::CHUNK_SIZE,
    modules::Module,
    range::Range,
    upc::{Bit, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
};

use super::coords::{CellCoord, ChunkCoord, LocalCoord, LOG_CHUNK_SIZE};

#[derive(Default, Serialize, Deserialize)]
pub struct Buffer {
    chunks: HashMap<ChunkCoord, BufferChunk>,
    transact_chunks: HashMap<ChunkCoord, BufferChunk>,
    modules: HashMap<CellCoord, Rc<RefCell<Module>>>,
    transact_modules: HashMap<CellCoord, Option<Rc<RefCell<Module>>>>,
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
}

#[derive(Serialize, Deserialize)]
struct BufferSnapshot {
    pub chunks: HashMap<ChunkCoord, Option<BufferChunk>>,
    pub modules: HashMap<CellCoord, Option<Rc<RefCell<Module>>>>,
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

    pub fn get_module<T>(&self, c: T) -> Option<Rc<RefCell<Module>>>
    where
        T: Into<CellCoord>,
    {
        let cell_coord: CellCoord = c.into();
        if let Some(transact_module) = self.transact_modules.get(&cell_coord) {
            transact_module.clone()
        } else {
            self.modules.get(&cell_coord).cloned()
        }
    }

    pub fn get_modules(&self) -> Vec<Rc<RefCell<Module>>> {
        // Return all module from transact_modules that aren't None, and return all modules from
        // `modules` that aren't in transact_module.
        self.transact_modules
            .values()
            .filter(|m| m.is_some())
            .map(|m| m.clone().unwrap())
            .chain(
                self.modules
                    .iter()
                    .filter(|(c, _)| !self.transact_modules.contains_key(c))
                    .map(|(_, m)| m.clone()),
            )
            .collect()
    }

    /// Returns an iterator for only committed chunk data (ignoring any pending transactions).
    pub fn get_base_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &BufferChunk)> {
        self.chunks.iter()
    }

    pub fn get_base_modules(&self) -> impl Iterator<Item = &Rc<RefCell<Module>>> {
        self.modules.values()
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
        self.transact = true;
    }

    pub fn transaction_abort(&mut self) {
        self.transact = false;
        self.transact_chunks.clear();
    }

    pub fn transaction_commit(&mut self, push_undo_stack: bool) {
        if push_undo_stack && (self.transact_chunks.len() > 0 || self.transact_modules.len() > 0) {
            self.undo_stack
                .push(self.snapshot(self.transact_chunks.keys(), self.transact_modules.keys()));
            self.redo_stack.clear();
        }

        self.transact = false;

        for (coord, chunk) in self.transact_chunks.drain() {
            if chunk.cell_count == 0 {
                self.chunks.remove(&coord);
            } else {
                self.chunks.insert(coord, chunk);
            }
        }

        for (coord, module) in self.transact_modules.drain() {
            if let Some(module) = module {
                self.modules.insert(coord, module);
            } else {
                self.modules.remove(&coord);
            }
        }
    }

    pub fn transaction_undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack
                .push(self.snapshot(snapshot.chunks.keys(), snapshot.modules.keys()));
            self.snapshot_apply(snapshot);
        }
    }

    pub fn transaction_redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack
                .push(self.snapshot(snapshot.chunks.keys(), snapshot.modules.keys()));
            self.snapshot_apply(snapshot);
        }
    }

    pub fn transact_set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<CellCoord>,
    {
        let coord: CellCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        let chunk = if let Some(chunk) = self.transact_chunks.get_mut(&chunk_coord) {
            chunk
        } else {
            let chunk = self.get_chunk(chunk_coord).cloned().unwrap_or_default();
            self.transact_chunks.insert(chunk_coord, chunk);
            self.transact_chunks.get_mut(&chunk_coord).unwrap()
        };

        chunk.set_cell(coord, cell);
    }

    pub fn transact_set_chunk<T>(&mut self, c: T, chunk: BufferChunk)
    where
        T: Into<ChunkCoord>,
    {
        let chunk_coord: ChunkCoord = c.into();
        self.transact_chunks.insert(chunk_coord, chunk);
    }

    /// Set the module and updates associated cells when the module places an IO pin.
    pub fn transact_set_module(&mut self, module: Rc<RefCell<Module>>) {
        let cell_coord = module.borrow().get_anchor().root;

        // Remove the previous module first (clears out the IO pins).
        self.transact_remove_module(cell_coord);

        // Set the pins for the new module.
        for pin in module.borrow().get_pins() {
            let mut upc = self.get_cell(pin);
            upc.set_bit(Bit::IO);
            self.transact_set_cell(pin, upc);
        }

        // Finally add the module.
        self.transact_modules.insert(cell_coord, Some(module));
    }

    pub fn transact_remove_module<T>(&mut self, c: T)
    where
        T: Into<CellCoord>,
    {
        let cell_coord: CellCoord = c.into();

        if let Some(pins) = self.get_module(cell_coord).map(|m| m.borrow().get_pins()) {
            for pin in pins {
                let mut upc = self.get_cell(pin);
                upc.clear_bit(Bit::IO);
                self.transact_set_cell(pin, upc);
            }
        }

        self.transact_modules.insert(cell_coord, None);
    }

    pub fn clone_cells_range(&self, range: Range) -> Buffer {
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

    fn snapshot<'a, TChunk, TCell>(&'a self, chunks: TChunk, modules: TCell) -> BufferSnapshot
    where
        TChunk: Iterator<Item = &'a ChunkCoord>,
        TCell: Iterator<Item = &'a CellCoord>,
    {
        BufferSnapshot {
            chunks: HashMap::from_iter(
                chunks.map(|coord| (*coord, self.chunks.get(coord).cloned())),
            ),
            modules: HashMap::from_iter(
                modules.map(|coord| (*coord, self.modules.get(coord).cloned())),
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

        for (coord, module) in snapshot.modules {
            if let Some(module) = module {
                self.modules.insert(coord, module);
            } else {
                self.modules.remove(&coord);
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
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
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
            cell_count: Default::default(),
        }
    }
}
