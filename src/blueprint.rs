use crate::{
    buffer::{Buffer, BufferChunk},
    coords::ChunkCoord,
    modules::ModuleData,
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Blueprint {
    chunks: Vec<CellChunk>,
    modules: Vec<ModuleData>,
}

#[derive(Serialize, Deserialize)]
struct CellChunk {
    chunk_coord: ChunkCoord,
    cells: Vec<Cell>,
}

#[derive(Serialize, Deserialize)]
struct Cell {
    upc_idx: u16,
    flags_1: u8,
    flags_2: u8,
}

impl From<&Buffer> for Blueprint {
    fn from(buffer: &Buffer) -> Self {
        let mut chunks = Vec::new();

        for (chunk_coord, chunk) in buffer.get_base_chunks() {
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
                cells.push(Cell {
                    upc_idx: (i / UPC_BYTE_LEN) as u16,
                    flags_1: r,
                    flags_2: g,
                });
            }

            chunks.push(CellChunk {
                chunk_coord: *chunk_coord,
                cells,
            });
        }

        Self {
            chunks,
            modules: buffer.get_base_modules().map(|m| m.clone()).collect(),
        }
    }
}

impl From<&Blueprint> for Buffer {
    fn from(blueprint: &Blueprint) -> Self {
        let mut buffer = Buffer::default();
        buffer.transaction_begin();

        for chunk in blueprint.chunks.iter() {
            let mut buffer_chunk = BufferChunk::default();
            buffer_chunk.cell_count = chunk.cells.len();
            // buffer_chunk.modules = chunk.modules.clone();

            for cell in chunk.cells.iter() {
                let mut upc_bytes = vec![0u8; UPC_BYTE_LEN];
                upc_bytes[0] = cell.flags_1;
                upc_bytes[1] = cell.flags_2;

                let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;
                buffer_chunk.cells[byte_idx] = cell.flags_1;
                buffer_chunk.cells[byte_idx + 1] = cell.flags_2;
            }

            buffer.transact_set_chunk(chunk.chunk_coord, buffer_chunk);
        }

        // Technically we could just blit these in (because the pins are already set) but I'm lazy.
        for module in blueprint.modules.iter() {
            buffer.transact_set_module(module.clone());
        }

        buffer.transaction_commit(false);
        buffer
    }
}
