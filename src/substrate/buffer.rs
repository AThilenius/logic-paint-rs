use glam::{IVec2, UVec2};
use itertools::Itertools;
use wasm_bindgen::prelude::*;

use crate::{
    coords::{CellCoord, ChunkCoord, LocalCoord, CHUNK_SIZE, LOG_CHUNK_SIZE},
    socket::Socket,
    upc::{Metal, NormalizedCell, Placement, Silicon, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
    utils::Selection,
};

/// Buffers are an infinite grid of cells, where each cell is 4 bytes. Things are split into
/// chunks, where each chunk stores a simple Vec<u8>, and Chunks are indexed by their chunk
/// coordinate on the infinite grid. Chunks with zero non-default cells take up no memory.
///
/// This struct is cheap to clone, as chunks are Copy-On-Write thanks to `im` HashMap. Sockets
/// however are cloned in their entirety, because they are relatively small.
#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct Buffer {
    #[wasm_bindgen(skip)]
    pub chunks: im::HashMap<ChunkCoord, BufferChunk>,
    #[wasm_bindgen(skip)]
    pub sockets: Vec<Socket>,
}

#[derive(Clone)]
pub struct BufferChunk {
    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    pub cells: Vec<u8>,

    /// How many cells are non-default.
    pub cell_count: usize,
}

#[wasm_bindgen]
impl Buffer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_cell(&self, cell_coord: CellCoord) -> UPC {
        let chunk_coord: ChunkCoord = cell_coord.into();
        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get_cell(cell_coord)
        } else {
            Default::default()
        }
    }

    pub fn set_cell(&mut self, cell_coord: CellCoord, cell: UPC) {
        let chunk_coord: ChunkCoord = cell_coord.into();

        self.chunks
            .entry(chunk_coord)
            .or_insert_with(|| Default::default())
            .set_cell(cell_coord, cell);
    }

    pub fn clone_selection(&self, selection: &Selection, anchor: CellCoord) -> Buffer {
        let mut buffer = Buffer::default();
        let ll = selection.lower_left.0;
        let ur = selection.upper_right.0;

        // Clone cells.
        for y in ll.y..ur.y {
            for x in ll.x..ur.x {
                let from_cell_coord = CellCoord(IVec2::new(x, y));
                let to_cell_coord = CellCoord(IVec2::new(x, y) - anchor.0);
                let cell = self.get_cell(from_cell_coord);
                buffer.set_cell(to_cell_coord, cell);
            }
        }

        buffer
    }

    pub fn paste_at(&mut self, cell_coord: CellCoord, buffer: &Buffer) {
        // While pasting values, find the bounding rect the target we are pasting into.
        let mut ll = IVec2::new(i32::MAX, i32::MAX);
        let mut ur = IVec2::new(i32::MIN, i32::MIN);

        // Paste cells
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
    }

    pub fn rotate_to_new(&self) -> Self {
        let mut buffer = Self::default();

        for (chunk_coord, chunk) in &self.chunks {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let c = local_coord.to_cell_coord(chunk_coord).0;

                    // Rotate coordinates around the origin.
                    buffer.set_cell(
                        CellCoord(IVec2::new(c.y, -c.x)),
                        chunk.get_cell(local_coord).rotate(),
                    );
                }
            }
        }

        // Todo: modules can't be rotated. That's not great UX though.

        buffer
    }

    pub fn mirror_to_new(&self) -> Self {
        let mut buffer = Self::default();

        for (chunk_coord, chunk) in &self.chunks {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let c = local_coord.to_cell_coord(chunk_coord).0;

                    // Mirror around the x axis (flip Y values).
                    buffer.set_cell(
                        CellCoord(IVec2::new(c.x, -c.y)),
                        chunk.get_cell(local_coord).mirror(),
                    );
                }
            }
        }

        // Todo: modules can't be mirrored. That's not great UX though.

        buffer
    }

    pub fn fix_all_cells(&mut self) {
        let chunk_coords = self.chunks.keys().cloned().collect_vec();
        for chunk_coord in chunk_coords {
            let chunk_first_cell = chunk_coord.first_cell_coord().0;
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    self.fix_cell(CellCoord(IVec2::new(x as i32, y as i32) + chunk_first_cell));
                }
            }
        }
    }

    pub fn cell_count(&self) -> usize {
        self.chunks.values().map(|c| c.cell_count).sum()
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

        if orig_upc == Default::default() {
            return;
        }

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
    pub fn set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
        let slice = &mut self.cells[idx..idx + UPC_BYTE_LEN];
        let existing = UPC::from_slice(slice);

        // Track cell count.
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
