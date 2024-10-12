use std::convert::TryFrom;

use wasm_bindgen::prelude::*;

use crate::substrate::buffer::Buffer;

use super::codec::EncodeV1;

#[derive(bincode::Encode, bincode::Decode)]
pub enum VersionWrapper {
    V1(EncodeV1),
}

#[wasm_bindgen]
impl Buffer {
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        bincode::encode_to_vec(
            VersionWrapper::V1(EncodeV1::from(self)),
            bincode::config::standard(),
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Buffer, JsValue> {
        let (version, _bytes_read): (VersionWrapper, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

        match version {
            VersionWrapper::V1(encode_v1) => {
                Buffer::try_from(encode_v1).map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }
}
