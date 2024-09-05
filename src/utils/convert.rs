use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    coords::ChunkCoord,
    substrate::buffer::{Buffer, BufferChunk},
    upc::LOG_UPC_BYTE_LEN,
};

#[derive(Serialize, Deserialize)]
struct Blueprint {
    chunks: HashMap<ChunkCoord, String>,
    #[allow(dead_code)]
    #[serde(skip)]
    modules: JsValue,
}

#[derive(Serialize, Deserialize)]
struct Cell {
    upc_idx: u16,
    flags_1: u8,
    flags_2: u8,
}

/// Convert a legacy blueprint JSON file into a Buffer (which can then be saved into the latest
/// format). Does not support modules, only the substrate is loaded.
#[wasm_bindgen]
pub fn import_legacy_blueprint(json_str: String) -> Result<Buffer, JsValue> {
    let blueprint: Blueprint =
        serde_json::from_str(&json_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut buffer = Buffer::default();

    for (chunk_coord, cells) in blueprint.chunks {
        let cells: Vec<Cell> = {
            if let Ok(bin) = STANDARD.decode(cells) {
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
            let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;

            // Old format used 8 bits on R, and 6 on G, both starting at MSB.
            // Later format uses 7 bits on each, moved down 1 bit
            buffer_chunk.cells[byte_idx] = (cell.flags_1 >> 1) & 0b0111_1111;
            buffer_chunk.cells[byte_idx + 1] = ((cell.flags_1 & 0b1) << 6) | (cell.flags_2 >> 2);
        }

        buffer.chunks.insert(chunk_coord, buffer_chunk);
    }

    Ok(buffer)
}
