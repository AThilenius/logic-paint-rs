use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    logic_paint_context::{LogicPaintContext, Msg as YewViewportMsg},
    session::SerdeSession,
};

#[wasm_bindgen]
pub struct LogicPaint {
    handle: AppHandle<LogicPaintContext>,
}

#[wasm_bindgen]
impl LogicPaint {
    #[wasm_bindgen(constructor)]
    pub fn new(root: Element) -> LogicPaint {
        LogicPaint {
            handle: yew::start_app_in_element::<LogicPaintContext>(root),
        }
    }

    pub fn set_session_from_string(&mut self, data: &str) -> Option<String> {
        if !data.starts_with("LPS1:") {
            return Some("String is not valid LogicPaint Session Version 1 data.".to_string());
        }

        let compressed_bytes = {
            if let Ok(compressed_bytes) = base64::decode(&data[8..]) {
                compressed_bytes
            } else {
                return Some(
                    "Data is not valid Base64. Are you missing some characters at the end?"
                        .to_string(),
                );
            }
        };

        let bytes = {
            if let Ok(bytes) = decompress_to_vec(&compressed_bytes) {
                bytes
            } else {
                return Some(
                    "Failed to decompress session data. Das ist nicht so gut...".to_string(),
                );
            }
        };

        let session = {
            let res = bincode::deserialize::<SerdeSession>(&bytes);

            if let Err(err) = res {
                return Some(err.to_string());
            }

            res.unwrap()
        };

        self.handle
            .send_message(YewViewportMsg::SetSession((&session).into()));

        None
    }

    pub fn get_session_as_string(&mut self) -> String {
        if let Some(component) = self.handle.get_component() {
            let bytes = bincode::serialize(&SerdeSession::from(&component.session)).unwrap_throw();
            let compressed_bytes = compress_to_vec(&bytes, 6);
            format!("LPS1:{}", base64::encode(compressed_bytes))
        } else {
            "".into()
        }
    }
}
