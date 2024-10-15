use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{coords::ChunkCoord, substrate::buffer::Buffer, upc::UPC};

#[derive(Serialize, Deserialize)]
struct Blueprint {
    chunks: HashMap<ChunkCoord, String>,
    #[allow(dead_code)]
    #[serde(skip)]
    modules: JsValue,
}

#[derive(bincode::Decode)]
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
                if let Ok((cells, _)) = bincode::decode_from_slice(&bin, bincode::config::legacy())
                {
                    cells
                } else {
                    continue;
                }
            } else {
                continue;
            }
        };

        for cell in cells.iter() {
            // Chunks were 32 wide originally. We need to convert to cell coord first, as chunks
            // are larger.
            // Local coords
            let x = (cell.upc_idx & (32 - 1)) as i32;
            let y = (cell.upc_idx >> 5) as i32;
            // Cell coors, converted from the old 32x32 chunks
            let c_x = (chunk_coord.0.x << 5) + x;
            let c_y = (chunk_coord.0.y << 5) + y;

            buffer.set_cell(
                (c_x, c_y).into(),
                UPC::from_slice(&[cell.flags_1, cell.flags_2, 0, 0]),
            );
        }
    }

    Ok(buffer)
}
