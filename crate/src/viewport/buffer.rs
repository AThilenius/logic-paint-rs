use std::collections::HashMap;

use glam::{IVec2, UVec2};

use crate::{
    coords::{CellCoord, ChunkCoord, LocalCoord, CHUNK_SIZE, LOG_CHUNK_SIZE},
    modules::{Pin, RootedModule},
    upc::{Bit, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
    utils::Selection,
};

#[derive(Default, Clone)]
pub struct Buffer {
    pub chunks: HashMap<ChunkCoord, BufferChunk>,
    pub rooted_modules: HashMap<CellCoord, RootedModule>,
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

    pub fn set_modules<'a, T>(&mut self, rooted_modules: T)
    where
        T: IntoIterator<Item = RootedModule>,
    {
        for rooted_module in rooted_modules.into_iter() {
            let anchor_coord = rooted_module.root;
            let mut upc = self.get_cell(anchor_coord);
            upc.set_bit(Bit::MODULE_ROOT);
            self.set_cell(anchor_coord, upc);

            for Pin { coord_offset, .. } in rooted_module.module.borrow().get_pins() {
                let pin_coord = coord_offset.to_cell_coord(anchor_coord);
                let mut upc = self.get_cell(pin_coord);
                upc.set_bit(Bit::IO);
                self.set_cell(pin_coord, upc);
            }

            self.rooted_modules
                .insert(anchor_coord, rooted_module.clone());
        }
    }

    pub fn clone_selection(&self, selection: &Selection, root: CellCoord) -> Buffer {
        let mut buffer = Buffer::default();
        let ll = selection.lower_left.0;
        let ur = selection.upper_right.0;

        for y in ll.y..ur.y {
            for x in ll.x..ur.x {
                let from_cell_coord = CellCoord(IVec2::new(x, y));
                let to_cell_coord = CellCoord(IVec2::new(x, y) - root.0);
                let cell = self.get_cell(from_cell_coord);
                buffer.set_cell(to_cell_coord, cell);
            }
        }

        // Then test and copy each module root that is in the selection
        buffer.set_modules(
            self.rooted_modules
                .values()
                .filter(|m| selection.test_cell_in_selection(m.root))
                .map(|m| {
                    let mut module = m.clone();
                    module.root.0 -= root.0;
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

        self.set_modules(buffer.rooted_modules.values().cloned());
    }

    pub fn clock_modules(&mut self, time: f64) {
        for (_, rooted_module) in self.rooted_modules.iter_mut() {
            rooted_module.module.borrow_mut().clock(time);
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

        // ModuleRoot and IO bits cannot be un-set. (They are unset by creating an entirely new
        // Buffer). So make sure Cell sets them if the previous cell had them.
        if existing.get_bit(Bit::IO) {
            cell.set_bit(Bit::IO);
        }

        if existing.get_bit(Bit::MODULE_ROOT) {
            cell.set_bit(Bit::MODULE_ROOT);
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
