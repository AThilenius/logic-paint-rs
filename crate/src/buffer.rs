use std::collections::HashMap;

use crate::{
    coords::CHUNK_SIZE,
    modules::{AnchoredModule, Pin},
    upc::{Bit, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
};

use super::coords::{CellCoord, ChunkCoord, LocalCoord, LOG_CHUNK_SIZE};

#[derive(Default, Clone)]
pub struct Buffer {
    pub chunks: HashMap<ChunkCoord, BufferChunk>,
    pub anchored_modules: HashMap<CellCoord, AnchoredModule>,
}

#[derive(Clone)]
pub struct BufferChunk {
    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<u8>,

    /// How many cells are non-default.
    pub cell_count: usize,
}

impl Buffer {
    pub fn get_cell<T>(&self, c: T) -> UPC
    where
        T: Into<CellCoord>,
    {
        let coord: CellCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();
        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get_cell(coord)
        } else {
            Default::default()
        }
    }

    pub fn set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<CellCoord>,
    {
        let coord: CellCoord = c.into();
        let chunk_coord: ChunkCoord = coord.into();

        self.chunks
            .entry(chunk_coord)
            .or_insert_with(|| Default::default())
            .set_cell(coord, cell);
    }

    pub fn set_modules<'a, T>(&mut self, anchored_modules: T)
    where
        T: IntoIterator<Item = AnchoredModule>,
    {
        for anchored_module in anchored_modules.into_iter() {
            let anchor_coord = anchored_module.anchor.root;

            for Pin { coord_offset, .. } in anchored_module.module.borrow().get_pins() {
                let pin_coord = coord_offset.to_cell_coord(anchor_coord);
                let mut upc = self.get_cell(pin_coord);
                upc.set_bit(Bit::IO);
                self.set_cell(pin_coord, upc);
            }

            self.anchored_modules
                .insert(anchor_coord, anchored_module.clone());
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

    // TODO: Bug here I think. Need to update count for any cells returned to default.
    pub fn clone_without_io_pins_set(&self) -> Self {
        let mut chunk = Self::default();

        for i in 0..(CHUNK_SIZE * CHUNK_SIZE) {
            let idx = i << LOG_UPC_BYTE_LEN;
            let src = &self.cells[idx..idx + UPC_BYTE_LEN];
            let target = &mut chunk.cells[idx..idx + UPC_BYTE_LEN];
            let mut cell = UPC::from_slice(src);
            cell.clear_bit(Bit::IO);
            target.copy_from_slice(&cell.0);
        }

        chunk
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
