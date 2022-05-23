use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    coords::CHUNK_SIZE,
    modules::{ModuleData, Pin},
    range::Range,
    upc::{Bit, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
};

use super::coords::{CellCoord, ChunkCoord, LocalCoord, LOG_CHUNK_SIZE};

#[derive(Default, Serialize, Deserialize)]
pub struct Buffer {
    chunks: HashMap<ChunkCoord, BufferChunk>,
    transact_chunks: HashMap<ChunkCoord, BufferChunk>,
    modules: HashMap<CellCoord, ModuleData>,
    transact_modules: HashMap<CellCoord, Option<ModuleData>>,
    transact: bool,
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
    pub modules: HashMap<CellCoord, Option<ModuleData>>,
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

    pub fn get_module<T>(&self, c: T) -> Option<ModuleData>
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

    pub fn get_modules(&self) -> Vec<ModuleData> {
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

    pub fn get_base_modules(&self) -> impl Iterator<Item = &ModuleData> {
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

    pub fn transaction_commit(&mut self) {
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
    pub fn transact_set_module(&mut self, module: ModuleData) {
        let cell_coord = module.get_anchor().root;

        // Remove the previous module first (clears out the IO pins).
        self.transact_remove_module(cell_coord);

        // Set the pins for the new module.
        for Pin { coord, .. } in module.get_pins() {
            let mut upc = self.get_cell(coord);
            upc.set_bit(Bit::IO);
            upc.set_bit(Bit::METAL);
            self.transact_set_cell(coord, upc);
        }

        // Finally add the module.
        self.transact_modules.insert(cell_coord, Some(module));
    }

    pub fn transact_remove_module<T>(&mut self, c: T)
    where
        T: Into<CellCoord>,
    {
        let cell_coord: CellCoord = c.into();

        if let Some(pins) = self.get_module(cell_coord).map(|m| m.get_pins()) {
            for Pin { coord, .. } in pins {
                let mut upc = self.get_cell(coord);
                upc.clear_bit(Bit::IO);
                upc.clear_bit(Bit::METAL);
                self.transact_set_cell(coord, upc);
            }
        }

        self.transact_modules.insert(cell_coord, None);
    }

    #[allow(dead_code)]
    pub fn clone_cells_range(&self, range: Range) -> Buffer {
        // TODO: This can be made much more efficient (broken iter over effected chunks), but I
        // probably don't care. It's only executed in human-time. -shrug-
        let mut buf = Buffer::default();
        buf.transaction_begin();
        for cell_coord in range.iter_cell_coords() {
            buf.transact_set_cell(cell_coord, self.get_cell(cell_coord));
        }
        buf.transaction_commit();
        buf
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
    pub fn set_cell<T>(&mut self, c: T, mut cell: UPC)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
        let slice = &mut self.cells[idx..idx + UPC_BYTE_LEN];
        let existing = UPC::from_slice(slice);

        // IO pins cannot be replaced with this call, so set IO bit if existing cell has an IO.
        if existing.get_bit(Bit::IO) {
            cell.set_bit(Bit::IO);
        }

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
