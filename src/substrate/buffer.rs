use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use glam::{IVec2, UVec2};
use wasm_bindgen::prelude::*;

use crate::{
    coords::{CellCoord, ChunkCoord, LocalCoord, CHUNK_CELL_COUNT, CHUNK_SIZE, LOG_CHUNK_SIZE},
    socket::Socket,
    upc::{Metal, NormalizedCell, Placement, Silicon, LOG_UPC_BYTE_LEN, UPC, UPC_BYTE_LEN},
    utils::Selection,
};

const CHUNK_BYTE_LEN: usize = CHUNK_CELL_COUNT * UPC_BYTE_LEN;

#[derive(Default, Clone)]
#[wasm_bindgen]
pub struct Buffer {
    /// The chunks that make up this buffer, stored as a dense list of Copy On Write byte arrays,
    /// each ready for blitting to the GPU. Each chunk tracks how many cells are non-default, and
    /// empty chunks are repurposed when a new chunk is needed (zero-allocation). Additionally,
    /// this vec if very cheap to clone, as the chunk data is stored behind a Rc RefCell. This
    /// means that you can clone a Buffer, update one cell, and only a single chunk will actually
    /// be copied (the one that was copy-on-write updated).
    #[wasm_bindgen(skip)]
    pub chunks: Vec<BufferChunk>,
    /// TODO: Copy On Write
    #[wasm_bindgen(skip)]
    pub sockets: Vec<Socket>,
}

#[derive(Clone)]
pub struct BufferChunk {
    /// The chunk coord. Used by a buffer to index chunks, but not used by the BufferChunk itself.
    pub chunk_coord: ChunkCoord,

    /// How many cells are non-default.
    pub cell_count: usize,

    /// 4-byte cells, in row-major order. Ready for blitting to the GPU.
    /// Stored in a Rc<RefCell<_>> to allow for COW semantics. This sllows Buffers to be very
    /// cheaply cloned, and mutations to require a minimum set of chunk copies.
    cells: Rc<RefCell<[u8; CHUNK_BYTE_LEN]>>,
}

#[wasm_bindgen]
impl Buffer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_cell(&self, cell_coord: CellCoord) -> UPC {
        let chunk_coord: ChunkCoord = cell_coord.into();
        for chunk in &self.chunks {
            if chunk.chunk_coord == chunk_coord {
                return chunk.get_cell(cell_coord);
            }
        }

        // Chunk isn't allocated
        Default::default()
    }

    pub fn set_cell(&mut self, cell_coord: CellCoord, cell: UPC) {
        let chunk_coord: ChunkCoord = cell_coord.into();

        // Existing chunks
        for chunk in &mut self.chunks {
            if chunk.chunk_coord == chunk_coord {
                chunk.set_cell(cell_coord, cell);
                return;
            }
        }

        // If a default UPC is being set, there is nothing else to do (no point in making an empty
        // chunk).
        if cell == Default::default() {
            return;
        }

        // See if we have an empty chunk we can repurpose
        for chunk in &mut self.chunks {
            if chunk.cell_count == 0 {
                chunk.chunk_coord = chunk_coord;
                chunk.set_cell(cell_coord, cell);
                return;
            }
        }

        // Otherwise allocate a new chunk and push it to the back.
        let mut chunk = BufferChunk::new(chunk_coord);
        chunk.set_cell(cell_coord, cell);
        self.chunks.push(chunk);
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
        for chunk in &buffer.chunks {
            let target_first_cell_offset = chunk.chunk_coord.first_cell_coord().0 + cell_coord.0;

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

        for chunk in &self.chunks {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let c = local_coord.to_cell_coord(&chunk.chunk_coord).0;

                    // Rotate coordinates around the origin.
                    buffer.set_cell(
                        CellCoord(IVec2::new(c.y, -c.x)),
                        chunk.get_cell(local_coord).rotate(),
                    );
                }
            }
        }

        buffer
    }

    pub fn mirror_to_new(&self) -> Self {
        let mut buffer = Self::default();

        for chunk in &self.chunks {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));
                    let c = local_coord.to_cell_coord(&chunk.chunk_coord).0;

                    // Mirror around the x axis (flip Y values).
                    buffer.set_cell(
                        CellCoord(IVec2::new(c.x, -c.y)),
                        chunk.get_cell(local_coord).mirror(),
                    );
                }
            }
        }

        buffer
    }

    pub fn fix_all_cells(&mut self) {
        let chunk_coords: Vec<ChunkCoord> = self.chunks.iter().map(|c| c.chunk_coord).collect();
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
        self.chunks.iter().map(|c| c.cell_count).sum()
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
    pub fn new(chunk_coord: ChunkCoord) -> Self {
        Self {
            chunk_coord,
            cells: Rc::new(RefCell::new([0_u8; CHUNK_BYTE_LEN])),
            cell_count: 0,
        }
    }

    #[inline(always)]
    pub fn get_cell<T>(&self, c: T) -> UPC
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
        UPC::from_slice(&self.cells.borrow()[idx..idx + UPC_BYTE_LEN])
    }

    #[inline(always)]
    pub fn set_cell<T>(&mut self, c: T, cell: UPC)
    where
        T: Into<LocalCoord>,
    {
        let coord: LocalCoord = c.into();
        let idx = (((coord.0.y << LOG_CHUNK_SIZE) + coord.0.x) as usize) << LOG_UPC_BYTE_LEN;
        let existing = UPC::from_slice(&self.cells.borrow()[idx..idx + UPC_BYTE_LEN]);

        // Nothing else to do if the existing and set cell are the same (no need to check for
        // ownership even; nothing was updated).
        if existing == cell {
            return;
        }

        // Track cell counts
        if existing == Default::default() && cell != Default::default() {
            self.cell_count += 1;
        } else if existing != Default::default() && cell == Default::default() {
            self.cell_count -= 1;
        }

        // Take ownership of the cell data and set it.
        let cells = self.get_cells_mut();
        let slice = &mut cells[idx..idx + UPC_BYTE_LEN];
        slice.copy_from_slice(&cell.0);
    }

    pub fn get_cells(&self) -> Ref<[u8; CHUNK_BYTE_LEN]> {
        self.cells.borrow()
    }

    pub fn get_cells_mut(&mut self) -> &mut [u8; CHUNK_BYTE_LEN] {
        Rc::make_mut(&mut self.cells).get_mut()
    }
}
