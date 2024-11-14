use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{
    coords::{ChunkCoord, CHUNK_CELL_COUNT},
    substrate::buffer::{Buffer, BufferChunk},
    upc::UPC_BYTE_LEN,
};

#[derive(bincode::Encode, bincode::Decode)]
pub enum VersionWrapper {
    // Brotli compressed chunk data, using BROTLI_CHANNELS bytes per cell with 128 cell wide
    // chunks. If either of these things change, the format version will need to be bumped.
    V1(BrotliBuffer),
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct BrotliBuffer {
    channels: u32,
    chunks: Vec<BrotliChunk>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct BrotliChunk {
    chunk_x: i32,
    chunk_y: i32,
    cell_count: u32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl Buffer {
    pub fn to_base64_string(&self) -> Result<String, JsValue> {
        let bytes = self.to_bytes()?;
        Ok(STANDARD.encode(bytes))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        const BROTLI_CHANNELS: usize = 2;
        let mut brotli_buffer = BrotliBuffer {
            chunks: Vec::new(),
            channels: BROTLI_CHANNELS as u32,
        };

        for (chunk_coord, chunk) in &self.chunks {
            let mut brotli_image = [0_u8; CHUNK_CELL_COUNT * BROTLI_CHANNELS];
            for i in 0..CHUNK_CELL_COUNT {
                for j in 0..BROTLI_CHANNELS {
                    brotli_image[i * BROTLI_CHANNELS + j] = chunk.cells[i * UPC_BYTE_LEN + j];
                }
            }

            let mut writer = brotli::CompressorWriter::new(
                Vec::new(),
                CHUNK_CELL_COUNT * BROTLI_CHANNELS,
                7,
                22,
            );
            writer.write_all(&brotli_image).unwrap();
            writer.flush().unwrap();
            let data = writer.into_inner();

            brotli_buffer.chunks.push(BrotliChunk {
                chunk_x: chunk_coord.0.x,
                chunk_y: chunk_coord.0.y,
                cell_count: chunk.cell_count as u32,
                data,
            });
        }

        // Bincode the frames
        let final_bytes = bincode::encode_to_vec(
            VersionWrapper::V1(brotli_buffer),
            bincode::config::standard(),
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(final_bytes)
    }

    pub fn from_base64_string(base_64_string: &str) -> Result<Buffer, JsValue> {
        let bytes = STANDARD
            .decode(base_64_string)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Buffer::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Buffer, JsValue> {
        let (version, _bytes_read): (VersionWrapper, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

        match version {
            VersionWrapper::V1(brotli_buffer) => {
                let channels = brotli_buffer.channels as usize;
                let mut buffer = Buffer::new();
                let mut brotli_image = vec![0_u8; CHUNK_CELL_COUNT * channels];

                for chunk in &brotli_buffer.chunks {
                    let mut reader = brotli::Decompressor::new(&chunk.data[..], CHUNK_CELL_COUNT);
                    match reader.read(&mut brotli_image[..]) {
                        Ok(read_size) => {
                            if read_size != CHUNK_CELL_COUNT * channels {
                                return Err(JsValue::from_str(&format!(
                                    "expected {} bytes but {} bytes were read",
                                    CHUNK_CELL_COUNT * channels,
                                    read_size
                                )));
                            }
                        }
                        Err(e) => {
                            return Err(JsValue::from_str(&format!("{}", e)));
                        }
                    }

                    // Convert to standard chunk
                    let mut cells = vec![0_u8; CHUNK_CELL_COUNT * UPC_BYTE_LEN];
                    for i in 0..CHUNK_CELL_COUNT {
                        for j in 0..channels {
                            cells[i * UPC_BYTE_LEN + j] = brotli_image[i * channels + j];
                        }
                    }

                    buffer.chunks.insert(
                        ChunkCoord((chunk.chunk_x, chunk.chunk_y).into()),
                        BufferChunk {
                            cells,
                            cell_count: chunk.cell_count as usize,
                        },
                    );
                }

                Ok(buffer)
            }
        }
    }
}
