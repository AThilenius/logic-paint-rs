use std::convert::TryFrom;

use crate::{
    coords::{ChunkCoord, CHUNK_CELL_COUNT, CHUNK_SIZE},
    substrate::buffer::{Buffer, BufferChunk},
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
};

// V1 JSON: 180 KiB
// V1 Bincode: 152 KiB
// V2 Single-axis RLN: 85 KiB

// ==== Common ====================================================================================
#[derive(thiserror::Error, Debug)]
pub enum CodecError {
    #[error("index out of bounds: `{0}`")]
    IndexOutOfBounds(String),

    #[error("too many elements provided: `{0}`")]
    BufferOverrun(String),
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

// ==== V2 - Single-axis run length coding ==========================================================

#[derive(bincode::Encode, bincode::Decode)]
pub struct EncodeV2 {
    chunks: Vec<ChunksV2>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct ChunksV2 {
    chunk_x: i32,
    chunk_y: i32,
    si: Vec<Run>,
    metal: Vec<Run>,
    // Offsets between each via, in row-major order
    via_offsets: Vec<u32>,
}

#[derive(bincode::Encode, bincode::Decode)]
struct Run {
    // Var-length
    length: u32,
    // Fixed length
    data: u8,
}

impl Run {
    pub fn decode(runs: &Vec<Run>) -> Result<[u8; CHUNK_CELL_COUNT], CodecError> {
        let mut arr = [0_u8; CHUNK_CELL_COUNT];

        // Check overflow
        if runs.iter().map(|run| run.length).sum::<u32>() > CHUNK_CELL_COUNT as u32 {
            return Err(CodecError::BufferOverrun("run is too long".into()));
        }

        let mut i = 0;
        for run in runs {
            for i in i..i + run.length {
                if run.data != 0 {
                    arr[i as usize] = run.data;
                }
            }

            i += run.length;
        }

        Ok(arr)
    }

    pub fn encode(arr: Vec<u8>) -> Vec<Run> {
        let mut runs = vec![];
        let mut run = Run {
            data: arr[0],
            length: 0,
        };

        for i in 0..CHUNK_CELL_COUNT {
            if run.data == arr[i] {
                run.length += 1;
            } else {
                runs.push(run);
                run = Run {
                    data: arr[i],
                    length: 1,
                };
            }
        }

        // The last run can be elided if it's empty
        if run.data != 0 {
            runs.push(run);
        }

        runs
    }
}

impl From<&Buffer> for EncodeV2 {
    fn from(buffer: &Buffer) -> Self {
        let mut chunks = Vec::new();

        for (chunk_coord, chunk) in &buffer.chunks {
            let si = Run::encode(chunk.cells.iter().step_by(UPC_BYTE_LEN).cloned().collect());
            let metal = Run::encode(
                chunk
                    .cells
                    .iter()
                    .skip(1)
                    .step_by(UPC_BYTE_LEN)
                    // Remove vias
                    .map(|metal| metal & !(1 << 2))
                    .collect(),
            );

            let mut via_offsets = vec![];
            let mut last_i = 0;
            for i in 0..CHUNK_CELL_COUNT {
                let upc_i = i << LOG_UPC_BYTE_LEN;
                if chunk.cells[upc_i + 1] & (1 << 2) != 0 {
                    via_offsets.push((i - last_i) as u32);
                    last_i = i;
                }
            }

            chunks.push(ChunksV2 {
                chunk_x: chunk_coord.0.x,
                chunk_y: chunk_coord.0.y,
                si,
                metal,
                via_offsets,
            });
        }

        EncodeV2 { chunks }
    }
}

impl TryFrom<EncodeV2> for Buffer {
    type Error = CodecError;

    fn try_from(data: EncodeV2) -> Result<Self, Self::Error> {
        let mut buffer = Buffer::default();

        for chunk in data.chunks {
            let si = Run::decode(&chunk.si)?;
            let mut metal = Run::decode(&chunk.metal)?;

            // Convert via offsets to absolute index, and write them to the metal layer
            let mut i = 0;
            for offset in chunk.via_offsets {
                i += offset as usize;
                metal[i] |= 1 << 2;
            }

            let mut buffer_chunk = BufferChunk::default();

            for i in 0..CHUNK_CELL_COUNT {
                let upc_i = i << LOG_UPC_BYTE_LEN;
                buffer_chunk.cells[upc_i] = si[i];
                buffer_chunk.cells[upc_i + 1] = metal[i];
                if si[i] | metal[i] != 0 {
                    buffer_chunk.cell_count += 1;
                }
            }

            let chunk_coord = ChunkCoord((chunk.chunk_x, chunk.chunk_y).into());
            buffer.chunks.insert(chunk_coord, buffer_chunk);
        }

        Ok(buffer)
    }
}
