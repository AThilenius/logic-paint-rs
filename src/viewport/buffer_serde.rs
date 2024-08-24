use std::collections::HashMap;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use base64::prelude::*;

use crate::{
    coords::{CellCoord, ChunkCoord},
    modules::ConcreteModule,
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
    viewport::buffer::{Buffer, BufferChunk},
};

#[derive(Serialize, Deserialize)]
struct BufferSerdeV1 {
    chunks: HashMap<ChunkCoord, String>,
    modules: HashMap<CellCoord, ConcreteModule>,
}

#[derive(Serialize, Deserialize)]
struct Cell {
    upc_idx: u16,
    flags_1: u8,
    flags_2: u8,
}

impl Serialize for Buffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let buffer_serde_v1 = {
            let mut chunks = HashMap::new();

            for (chunk_coord, chunk) in &self.chunks {
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
                    chunks.insert(
                        chunk_coord.clone(),
                        BASE64_STANDARD.encode(
                            bincode::serialize(&cells).expect("Failed to bincode serialize cells"),
                        ),
                    );
                }
            }

            BufferSerdeV1 {
                chunks,
                modules: self.modules.clone(),
            }
        };
        
        buffer_serde_v1.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Buffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buffer_serde_v1 = BufferSerdeV1::deserialize(deserializer)?;
        
        let mut buffer = Buffer::default();

        for (chunk_coord, cells) in buffer_serde_v1.chunks {
            let cells: Vec<Cell> = {
                if let Ok(bin) = BASE64_STANDARD.decode(cells) {
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

            buffer.chunks.insert(chunk_coord, buffer_chunk);
        }

        // Set the modules.
        buffer.modules = buffer_serde_v1.modules;

        Ok(buffer)
    }
}
