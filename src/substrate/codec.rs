use std::convert::TryFrom;

use crate::{
    coords::{ChunkCoord, CHUNK_SIZE},
    substrate::buffer::{Buffer, BufferChunk},
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
};

// ==== Common ====================================================================================
#[derive(thiserror::Error, Debug)]
pub enum CodecError {
    #[error("index out of bounds `{0}`")]
    IndexOutOfBounds(String),
}

// ==== V1 - Individually addressed cells =========================================================
// Not terribly compact, but nice and easy to implement. I'll switch to a fancy Run Length Coding
// with multi-layer support some day.
#[derive(bincode::Encode, bincode::Decode)]
pub struct EncodeV1 {
    chunks: Vec<ChunksV1>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct ChunksV1 {
    chunk_x: i32,
    chunk_y: i32,
    cells: Vec<CellV1>,
}

#[derive(bincode::Encode, bincode::Decode)]
struct CellV1 {
    upc_idx: u16,
    flags_1: u8,
    flags_2: u8,
}

impl From<&Buffer> for EncodeV1 {
    fn from(buffer: &Buffer) -> Self {
        let mut chunks = Vec::new();

        for (chunk_coord, chunk) in &buffer.chunks {
            let mut cells = Vec::new();

            for i in (0..chunk.cells.len()).step_by(UPC_BYTE_LEN) {
                // Skip empty cells.
                let r = chunk.cells[i];
                let g = chunk.cells[i + 1];

                if r == 0 && g == 0 {
                    continue;
                }

                // We simply truncate the IO flag from the UPC format (ie just grab the first 16
                // bits of it).
                cells.push(CellV1 {
                    upc_idx: (i / UPC_BYTE_LEN) as u16,
                    flags_1: r,
                    flags_2: g,
                });
            }

            if cells.len() > 0 {
                chunks.push(ChunksV1 {
                    chunk_x: chunk_coord.0.x,
                    chunk_y: chunk_coord.0.y,
                    cells,
                });
            }
        }

        EncodeV1 { chunks }
    }
}

impl TryFrom<EncodeV1> for Buffer {
    type Error = CodecError;

    fn try_from(data: EncodeV1) -> Result<Self, Self::Error> {
        let mut buffer = Buffer::default();

        for chunk in data.chunks {
            let mut buffer_chunk = BufferChunk::default();
            let chunk_coord = ChunkCoord((chunk.chunk_x, chunk.chunk_y).into());
            buffer_chunk.cell_count = chunk.cells.len();

            for cell in &chunk.cells {
                let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;

                if byte_idx + 1 >= UPC_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE {
                    return Err(CodecError::IndexOutOfBounds(format!(
                        "chunk coord {:#?} has byte index {}",
                        chunk_coord, byte_idx
                    )));
                }

                buffer_chunk.cells[byte_idx] = cell.flags_1;
                buffer_chunk.cells[byte_idx + 1] = cell.flags_2;
            }

            buffer.chunks.insert(chunk_coord, buffer_chunk);
        }

        Ok(buffer)
    }
}

// ==== V2 - Dual-axis run length coding ==========================================================
// Run Length Encoding (RLE) used to highly compress a chunk in a buffer, specifically it describes the encoding strategy used for
// arbitrary (up to 7-bit) multi-layered 2D slices of 128x128.
//
// ## Run Length Storage
//
// Si, metal and vias are broken into independent 2D layers and are added back together during
// decoding. Each slice is made up of two sets of runs, one for row-major order, and another for
// column-major order. This allows long horizontal and vertical traces (which are common in
// designs) to be efficiently encoded.
//
// ### Decoding
//
// To decode a single layer, first the row-major runs are applied, then the column-major runs are
// applied, overwriting any previously set values from the row-major runs. It's that simple.
//
// ### Encoding
//
// Encoding is more tricky. Consider the "test cell" to be the next cell we are testing, to see if
// it should be included in the current run, or if a new run should be started.
//
// Row-Major Runs
// - The run is continued if either
//   - The run and test cell are like-typed
//   - The run and test cell are not like-typed AND the test cell is not like-typed the cell to
//     it's right
//     - This is the same check that column-major will do. In other words, we know the cell will be
//       included in the column-major runs, so we can treat it like a wild-card and continue the
//       run.
//     - Note that this also covers the case of transitioning from empty to non-empty cells
//
// ----------------
// ----------------
// ----------------
// ----2XXX?XXXX---
// ----------------
// ----------------
// ----------------
// Runs: 2. The 3rd run is empty and thus elided.
//
// Column-Major Runs
// - The run is continued if either
//   - The run and test cell are like-typed
//   - The current run is empty AND the test cell is like-type either the left or right cell
//     - Ie. we can elide cells already included in the row-major runs
//     - This is only possible for empty runs, as we can't add invalid cell in the column runs
// ----------------
// --------8-------
// --------7-------
// ---2----6----10-
// --------5-------
// --------4-------
// ----------------
// Runs: 10. The 11th run is empty and thus elided.
//
// The dual-axis encoding reduces the total number of runs from 26 down to 12
// for this simple feature. Additionally, the encoding is very computationally
// efficient when compared to a general purpose compression algorithm like gzip.
//
// Other alternatives were considered, like higher-order run-lengths (runs of
// runs) but the complexity is high and it doesn't fit well into protobuf's
// varlen format

#[derive(bincode::Encode, bincode::Decode)]
pub struct EncodeV2 {
    chunks: Vec<ChunksV2>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct ChunksV2 {
    chunk_x: i32,
    chunk_y: i32,
    si: Layer,
    metal: Layer,
    // Offsets between each via, in row-major order
    via_offsets: Vec<u32>,
}

#[derive(bincode::Encode, bincode::Decode)]
struct Layer {
    column_runs: Vec<Run>,
    row_runs: Vec<Run>,
}

#[derive(bincode::Encode, bincode::Decode)]
struct Run {
    // Var-length
    length: u32,
    // Fixed length
    data: u8,
}

impl From<&Buffer> for EncodeV2 {
    fn from(buffer: &Buffer) -> Self {
        let mut chunks = Vec::new();

        for (chunk_coord, chunk) in &buffer.chunks {
            for i in (0..chunk.cells.len()).step_by(UPC_BYTE_LEN) {
                // TODO
            }
        }

        EncodeV2 { chunks }
    }
}

impl TryFrom<EncodeV2> for Buffer {
    type Error = CodecError;

    fn try_from(data: EncodeV2) -> Result<Self, Self::Error> {
        let mut buffer = Buffer::default();

        for chunk in data.chunks {
            let mut buffer_chunk = BufferChunk::default();
            let chunk_coord = ChunkCoord((chunk.chunk_x, chunk.chunk_y).into());

            let mut i = 0;
            let mut cell_count = 0;

            for run in &chunk.row_runs {
                if i + run.length > 

                let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;

                if byte_idx + 1 >= UPC_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE {
                    return Err(CodecError::IndexOutOfBounds(format!(
                        "chunk coord {:#?} has byte index {}",
                        chunk_coord, byte_idx
                    )));
                }

                buffer_chunk.cells[byte_idx] = cell.flags_1;
                buffer_chunk.cells[byte_idx + 1] = cell.flags_2;
            }

            buffer_chunk.cell_count = cell_count;
            buffer.chunks.insert(chunk_coord, buffer_chunk);
        }

        Ok(buffer)
    }
}
