use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    coords::{ChunkCoord, CHUNK_SIZE},
    substrate::buffer::{Buffer, BufferChunk},
    upc::{LOG_UPC_BYTE_LEN, UPC_BYTE_LEN},
};

#[derive(Serialize, Deserialize)]
pub struct V2Data {
}

#[derive(Serialize, Deserialize)]
struct Layer {

}

struct Run {
    data: u16,
    length: 
}

pub fn serialize_v2(buffer: &Buffer, writer: &mut Vec<u8>) -> Result<(), JsValue> {
    let mut chunks = HashMap::new();

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
            chunks.insert(
                chunk_coord.clone(),
                bincode::serialize(&cells).map_err(|e| JsValue::from_str(&e.to_string()))?,
            );
        }
    }

    bincode::serialize_into(writer, &V1Data { chunks })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

pub fn deserialize_v2(bytes: &[u8]) -> Result<Buffer, JsValue> {
    let data: V1Data =
        bincode::deserialize(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut buffer = Buffer::default();

    for (chunk_coord, cells) in data.chunks {
        let cells: Vec<Cell> =
            bincode::deserialize(&cells).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut buffer_chunk = BufferChunk::default();
        buffer_chunk.cell_count = cells.len();

        for cell in cells.iter() {
            let byte_idx = (cell.upc_idx as usize) << LOG_UPC_BYTE_LEN;

            if byte_idx + 1 >= UPC_BYTE_LEN * CHUNK_SIZE * CHUNK_SIZE {
                return Err(JsValue::from_str(&format!(
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
