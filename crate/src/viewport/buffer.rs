use std::collections::HashMap;

use glam::{IVec2, UVec2};

use crate::{
    coords::{CellCoord, ChunkCoord, LocalCoord, CHUNK_SIZE, LOG_CHUNK_SIZE},
    log,
    modules::{Pin, RootedModule},
    upc::{Bit, Metal, NormalizedCell, Placement, Silicon, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
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
        // While pasting values, find the bounding rect the target we are pasting into.
        let mut ll = IVec2::new(i32::MAX, i32::MAX);
        let mut ur = IVec2::new(i32::MIN, i32::MIN);

        for (chunk_coord, chunk) in &buffer.chunks {
            let target_first_cell_offset = chunk_coord.first_cell_coord().0 + cell_coord.0;

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let cell = chunk.get_cell(local_coord);

                    if cell == Default::default() {
                        continue;
                    }

                    let target_cell_coord =
                        CellCoord(target_first_cell_offset + IVec2::new(x as i32, y as i32));

                    // Tracking bounding rect.
                    ll = IVec2::min(ll, target_cell_coord.0);
                    ur = IVec2::max(ur, target_cell_coord.0);

                    self.set_cell(target_cell_coord, cell);
                }
            }
        }

        // Then go through the outline of the bounding rect and fix any broken cells.
        for x in ll.x..(ur.x + 1) {
            self.fix_cell(CellCoord(IVec2::new(x, ll.y)));
            self.fix_cell(CellCoord(IVec2::new(x, ur.y)));
        }

        for y in ll.y..(ur.y + 1) {
            self.fix_cell(CellCoord(IVec2::new(ll.x, y)));
            self.fix_cell(CellCoord(IVec2::new(ur.x, y)));
        }

        self.set_modules(buffer.rooted_modules.values().cloned());
    }

    pub fn clock_modules(&mut self, time: f64) {
        for (_, rooted_module) in self.rooted_modules.iter_mut() {
            rooted_module.module.borrow_mut().clock(time);
        }
    }

    fn fix_cell(&mut self, cell_coord: CellCoord) {
        // Follow broken connection directions and connect them, if able. The following
        // connections will be made (every other connection will be dropped):
        // - Metal -> metal
        // Both NP and MOSFET, take the type trying to connect (N/P):
        // N : NP(n)
        // P : NP(p)
        // N : MOSFET(npn) if ec_in_line_with_dir
        // N : MOSFET(pnp) if gate_in_line_with_dir
        // P : MOSFET(pnp) if ec_in_line_with_dir
        // P : MOSFET(npn) if gate_in_line_with_dir
        //
        // Otherwise the connection is culled.

        let orig_upc = self.get_cell(cell_coord);
        let mut cell: NormalizedCell = orig_upc.into();

        // Handle metal (which is simple).
        if let Metal::Trace { placement: pl, .. } = &mut cell.metal {
            for dir in pl.cardinal_vectors() {
                let n: NormalizedCell = self.get_cell(CellCoord(cell_coord.0 + dir)).into();

                if let Metal::Trace {
                    placement: n_pl, ..
                } = n.metal
                {
                    if !n_pl.has_cardinal(-dir) {
                        // Cannot make the connection, so remove the placement.
                        pl.clear_cardinal(dir);
                    }
                } else {
                    pl.clear_cardinal(dir);
                }
            }
        }

        // Returns a new Placement type that has only valid connections for the is type in the given
        // directions.
        let check_si_pl = |pl: Placement, is_n: bool| {
            let mut fixed_pl = Placement::default();

            for dir in pl.cardinal_vectors() {
                let n: NormalizedCell = self.get_cell(CellCoord(cell_coord.0 + dir)).into();

                // This matches the success-table in the comment above.
                match (is_n, n.si) {
                    (
                        true,
                        Silicon::NP {
                            is_n: true,
                            placement: n_pl,
                        },
                    )
                    | (
                        false,
                        Silicon::NP {
                            is_n: false,
                            placement: n_pl,
                        },
                    )
                    | (
                        true,
                        Silicon::Mosfet {
                            is_npn: true,
                            ec_placement: n_pl,
                            ..
                        },
                    )
                    | (
                        true,
                        Silicon::Mosfet {
                            is_npn: false,
                            gate_placement: n_pl,
                            ..
                        },
                    )
                    | (
                        false,
                        Silicon::Mosfet {
                            is_npn: true,
                            gate_placement: n_pl,
                            ..
                        },
                    )
                    | (
                        false,
                        Silicon::Mosfet {
                            is_npn: false,
                            ec_placement: n_pl,
                            ..
                        },
                    ) => {
                        if n_pl.has_cardinal(-dir) {
                            fixed_pl.set_cardinal(dir);
                        }
                    }
                    _ => {}
                }
            }

            fixed_pl
        };

        // Handle Si type
        match &mut cell.si {
            Silicon::NP { is_n, placement } => {
                *placement = check_si_pl(*placement, *is_n);
            }
            Silicon::Mosfet {
                is_npn,
                gate_placement,
                ec_placement,
                ..
            } => {
                *ec_placement = check_si_pl(*ec_placement, *is_npn);
                *gate_placement = check_si_pl(*gate_placement, !*is_npn);
            }
            _ => {}
        }

        // Write the cell back.
        let new_upc: UPC = cell.into();
        self.set_cell(cell_coord, new_upc);
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

    pub fn clone_without_io_pins_set(&self) -> Self {
        let mut chunk = Self::default();
        chunk.cell_count = self.cell_count;

        for i in 0..(CHUNK_SIZE * CHUNK_SIZE) {
            let idx = i << LOG_UPC_BYTE_LEN;
            let src = &self.cells[idx..idx + UPC_BYTE_LEN];
            let target = &mut chunk.cells[idx..idx + UPC_BYTE_LEN];
            let orig_cell = UPC::from_slice(src);

            let mut cell = orig_cell;
            cell.clear_bit(Bit::IO);
            cell.clear_bit(Bit::MODULE_ROOT);
            target.copy_from_slice(&cell.0);

            if orig_cell != Default::default() && cell == Default::default() {
                chunk.cell_count -= 1;
            }
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
