use std::convert::TryFrom;
use std::io::prelude::*;
use wasm_bindgen::prelude::*;

use crate::utils::convert::import_legacy_blueprint;
use crate::{substrate::buffer::Buffer, upc::UPC};

use super::codec::{EncodeV1, EncodeV2};

#[derive(bincode::Encode, bincode::Decode)]
pub enum VersionWrapper {
    V1(EncodeV1),
    V2(EncodeV2),
    V3(EncodeV3Header),
}

#[derive(bincode::Encode, bincode::Decode, Debug)]
pub struct EncodeV3Header {
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct BufferImage {
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl Buffer {
    pub fn run_test() {
        let json = include_str!("../../misc/cpu.lpbp").to_string();
        let buffer = import_legacy_blueprint(json).unwrap();

        let bytes = buffer.to_bytes().unwrap();
        let _buffer = Buffer::from_bytes(&bytes).unwrap();
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        let image = self.as_image();

        let data = bincode::encode_to_vec(
            VersionWrapper::V3(EncodeV3Header {
                width: image.width,
                height: image.height,
                offset_x: image.offset_x,
                offset_y: image.offset_y,
            }),
            bincode::config::standard(),
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Deflate impl
        // let mut e = DeflateEncoder::new(data, Compression::default());
        // e.write_all(&image.data)
        //     .map_err(|e| JsValue::from_str(&e.to_string()))?;
        // let bytes = e.finish().unwrap();

        // Brotli impl
        let mut writer = brotli::CompressorWriter::new(data, image.data.len(), 7, 22);
        writer.write_all(&image.data).unwrap();
        writer.flush().unwrap();
        let bytes = writer.into_inner();

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Buffer, JsValue> {
        let (version, bytes_read): (VersionWrapper, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

        match version {
            VersionWrapper::V1(encode_v1) => Buffer::try_from(encode_v1),
            VersionWrapper::V2(encode_v2) => Buffer::try_from(encode_v2),
            VersionWrapper::V3(encode_v3_header) => {
                // Deflate impl
                // let mut deflater = DeflateDecoder::new(Vec::new());
                // deflater.write_all(&bytes[bytes_read..]).unwrap();
                // let data = deflater.finish().unwrap();

                // Brotli impl
                let size = encode_v3_header.width * encode_v3_header.height * 2;
                let mut reader = brotli::Decompressor::new(&bytes[bytes_read..], size as usize);
                let mut data = vec![0_u8; size as usize];
                match reader.read(&mut data[..]) {
                    Ok(read_size) => {
                        if size != read_size as u32 {
                            panic!("expected {} bytes but {} bytes were read", size, read_size);
                        }
                    }
                    Err(e) => {
                        panic!("{}", e);
                    }
                }

                Ok(Buffer::from_image(BufferImage {
                    width: encode_v3_header.width,
                    height: encode_v3_header.height,
                    offset_x: encode_v3_header.offset_x,
                    offset_y: encode_v3_header.offset_y,
                    data,
                }))
            }
        }
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Buffer {
    // Returns 2 channel (2 byte per pixel) image of all the chunks
    pub fn as_image(&self) -> BufferImage {
        let x_min = self
            .chunks
            .keys()
            .map(|k| k.first_cell_coord().0.x)
            .min()
            .unwrap();
        let y_min = self
            .chunks
            .keys()
            .map(|k| k.first_cell_coord().0.y)
            .min()
            .unwrap();
        let x_max = self
            .chunks
            .keys()
            .map(|k| k.last_cell_coord().0.x)
            .max()
            .unwrap();
        let y_max = self
            .chunks
            .keys()
            .map(|k| k.last_cell_coord().0.y)
            .max()
            .unwrap();

        let width = (x_max - x_min).abs() as u32;
        let height = (y_max - y_min).abs() as u32;
        let offset_x = x_min;
        let offset_y = y_min;

        let mut data = vec![0_u8; (width * height * 2) as usize];

        for cell_y in y_min..y_max {
            for cell_x in x_min..x_max {
                let image_x = cell_x - x_min;
                let image_y = cell_y - y_min;
                let image_i = (image_y * (width as i32) + image_x) * 2;
                let cell = self.get_cell((cell_x, cell_y).into());

                data[image_i as usize + 0] = cell.0[0];
                data[image_i as usize + 1] = cell.0[1];
            }
        }

        BufferImage {
            width,
            height,
            offset_x,
            offset_y,
            data,
        }
    }

    pub fn from_image(image: BufferImage) -> Buffer {
        let mut buffer = Buffer::new();

        let mut i = 0;
        for cell_y in image.offset_y..(image.offset_y + image.height as i32) {
            for cell_x in image.offset_x..(image.offset_x + image.width as i32) {
                buffer.set_cell(
                    (cell_x, cell_y).into(),
                    UPC::from_slice(&[image.data[i], image.data[i + 1], 0, 0]),
                );
                i += 2;
            }
        }

        buffer
    }
}
