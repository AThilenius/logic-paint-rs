use std::collections::HashMap;

use glam::{IVec2, UVec2};

use crate::{
    coords::CHUNK_SIZE,
    modules::{AnchoredModule, Pin},
    upc::{Bit, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
    utils::Selection,
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

    pub fn clone_selection(&self, selection: &Selection, root: CellCoord) -> Buffer {
        let mut buffer = Buffer::default();

        for (chunk_coord, chunk) in &self.chunks {
            // None of the chunk overlaps, continue.
            if !selection.test_any_of_chunk_in_selection(*chunk_coord) {
                continue;
            }

            let ll = LocalCoord::from(CellCoord(IVec2::max(
                chunk_coord.first_cell_coord().0,
                selection.lower_left.0,
            )))
            .0;

            let ur = LocalCoord::from(CellCoord(IVec2::min(
                chunk_coord.last_cell_coord().0,
                selection.upper_right.0,
            )))
            .0 + UVec2::ONE;

            for y in ll.y..ur.y {
                for x in ll.x..ur.x {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let cell = chunk.get_cell(local_coord);

                    let target_cell_coord =
                        CellCoord(local_coord.to_cell_coord(chunk_coord).0 - root.0);

                    buffer.set_cell(target_cell_coord, cell);
                }
            }
        }

        // Then test and copy each module root that is in the selection
        buffer.set_modules(
            self.anchored_modules
                .values()
                .filter(|m| selection.test_cell_in_selection(m.anchor.root))
                .map(|m| {
                    let mut module = m.clone();
                    module.anchor.root.0 -= root.0;
                    module
                }),
        );

        buffer
    }

    pub fn paste_at(&mut self, cell_coord: CellCoord, buffer: &Buffer) {
        for (chunk_coord, chunk) in &buffer.chunks {
            let target_first_cell_offset = chunk_coord.first_cell_coord().0 + cell_coord.0;

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let cell = chunk.get_cell(local_coord);

                    if cell == Default::default() {
                        continue;
                    }

                    self.set_cell(
                        CellCoord(target_first_cell_offset + IVec2::new(x as i32, y as i32)),
                        cell,
                    );
                }
            }
        }

        self.set_modules(buffer.anchored_modules.values().cloned());
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

        // IO pins and module roots cannot be replaced with this call, so set IO/root bit if
        // existing cell has one.
        cell.set_bit_val(Bit::IO, existing.get_bit(Bit::IO));
        cell.set_bit_val(Bit::MODULE_ROOT, existing.get_bit(Bit::MODULE_ROOT));

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
            cell.clear_bit(Bit::MODULE_ROOT);
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
