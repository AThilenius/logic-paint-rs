use crate::{
    buffer::{Buffer, BufferChunk},
    coords::ChunkCoord,
    modules::ModuleSerde,
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
};

use glam::IVec2;
use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;

#[derive(Serialize, Deserialize)]
pub struct Blueprint {
    pub root_offset: Option<IVec2>,
    chunks: Option<Vec<CellChunk>>,
    modules: Option<Vec<ModuleSerde>>,
}

#[derive(Serialize, Deserialize)]
struct CellChunk {
    chunk_coord: ChunkCoord,
    cells: String,
}

#[derive(Serialize, Deserialize)]
struct Cell {
    upc_idx: u16,
    flags_1: u8,
    flags_2: u8,
}

impl Blueprint {
    pub fn into_buffer_from_partial(&self, existing_buffer: &Buffer) -> Option<Buffer> {
        if self.chunks.is_none() && self.modules.is_none() {
            return None;
        }

        let mut buffer = Buffer::default();

        if let Some(chunks) = &self.chunks {
            for chunk in chunks {
                let cells: Vec<Cell> = {
                    if let Ok(bin) = base64::decode(&chunk.cells) {
                        if let Ok(cells) = bincode::deserialize(&bin) {
                            cells
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                };

                let mut buffer_chunk = BufferChunk::default();
                buffer_chunk.cell_count = cells.len();

                for cell in cells.iter() {
                    let mut upc_bytes = vec![0u8; UPC_BYTE_LEN];
                    upc_bytes[0] = cell.flags_1;
                    upc_bytes[1] = cell.flags_2;

                    let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;
                    buffer_chunk.cells[byte_idx] = cell.flags_1;
                    buffer_chunk.cells[byte_idx + 1] = cell.flags_2;
                }

                buffer.chunks.insert(chunk.chunk_coord, buffer_chunk);
            }
        } else {
            // Copy from existing buffer, without IO pins.
            for (chunk_coord, buffer_chunk) in &existing_buffer.chunks {
                buffer
                    .chunks
                    .insert(*chunk_coord, buffer_chunk.clone_without_io_pins_set());
            }
        }

        // Set the modules. This will also (re)set the module IO pins.
        if let Some(modules) = &self.modules {
            buffer.set_modules(modules.iter().map(|s| s.instantiate()));
        } else {
            buffer.set_modules(existing_buffer.anchored_modules.values().cloned());
        }

        Some(buffer)
    }
}

impl From<&Buffer> for Blueprint {
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
                cells.push(Cell {
                    upc_idx: (i / UPC_BYTE_LEN) as u16,
                    flags_1: r,
                    flags_2: g,
                });
            }

            if cells.len() > 0 {
                chunks.push(CellChunk {
                    chunk_coord: *chunk_coord,
                    cells: base64::encode(bincode::serialize(&cells).unwrap_throw()),
                });
            }
        }

        Self {
            root_offset: None,
            chunks: Some(chunks),
            modules: Some(
                buffer
                    .anchored_modules
                    .values()
                    .map(|a| a.module_serde.clone())
                    .collect(),
            ),
        }
    }
}
