use glam::{IVec2, UVec2};
use im::HashSet;

use crate::{
    coords::{CellCoord, ChunkCoord, LocalCoord, CHUNK_SIZE},
    upc::UPC,
};

use super::buffer::Buffer;

/// Stores a sparse set of deltas, encoded as (CellCoord, Cell) tuples. It's not particularly
/// compact in-memory, but compresses just fine. Because cell diffs get added one chunk at a time,
/// a BufferDelta can also be efficiently applied to a Buffer.
pub struct BufferDelta {
    cell_deltas: Vec<CellDelta>,
}

pub struct CellDelta {
    /// The cell coord of this delta.
    cell_coord: CellCoord,
    /// The call's value in UPC format. This value will be copied without modification to the
    /// Buffer while being applied (aka zero values are 'clear' ops, non-zero values set the cell).
    cell: UPC,
}

impl BufferDelta {
    /// Create a delta from two buffers. `from` is the previous buffer, and `to` is the target
    /// buffer. In other words, the delta can be applied to `from` and will result in `to` as an
    /// output.
    pub fn new(from: &Buffer, to: &Buffer) -> Self {
        let mut cell_deltas = vec![];

        // Start by collecting a full set of chunk coords we will be visiting, this is a union of
        // from and to chunks.
        let chunk_coords: HashSet<ChunkCoord> = from
            .chunks
            .iter()
            .map(|c| c.chunk_coord)
            .chain(to.chunks.iter().map(|c| c.chunk_coord))
            .collect();

        for chunk_coord in chunk_coords {
            let from = from.chunks.iter().find(|c| c.chunk_coord == chunk_coord);
            let to = to.chunks.iter().find(|c| c.chunk_coord == chunk_coord);
            let first_cell_coord = chunk_coord.first_cell_coord();

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let local_coord = LocalCoord(UVec2::new(x as u32, y as u32));

                    match (from, to) {
                        (None, None) => unreachable!(),
                        (None, Some(to)) => {
                            let cell = to.get_cell(local_coord);
                            // Only encode newly set cells
                            if cell != Default::default() {
                                cell_deltas.push(CellDelta::new(first_cell_coord, x, y, cell));
                            }
                        }
                        (Some(from), None) => {
                            let cell = from.get_cell(local_coord);
                            // Encode all from cells as empty (they were all unset)
                            if cell != Default::default() {
                                cell_deltas.push(CellDelta::new(
                                    first_cell_coord,
                                    x,
                                    y,
                                    Default::default(),
                                ));
                            }
                        }
                        (Some(from), Some(to)) => {
                            let from = from.get_cell(local_coord);
                            let to = to.get_cell(local_coord);

                            // Encode the `to` cell anywhere that from != to (that they changed)
                            if from != to {
                                cell_deltas.push(CellDelta::new(first_cell_coord, x, y, to));
                            }
                        }
                    }
                }
            }
        }

        Self { cell_deltas }
    }

    /// Apply this delta to the `from` buffer, returning what would have been passed into the
    /// `to` arg of the `new` method.
    pub fn apply(&self, from: &Buffer) -> Buffer {
        let mut to = from.clone();

        for cell_delta in &self.cell_deltas {
            to.set_cell(cell_delta.cell_coord, cell_delta.cell);
        }

        to
    }
}

impl CellDelta {
    pub fn new(first_cell_in_chunk: CellCoord, x: usize, y: usize, cell: UPC) -> Self {
        let cell_coord = CellCoord(first_cell_in_chunk.0 + IVec2::new(x as i32, y as i32));
        Self { cell_coord, cell }
    }
}
