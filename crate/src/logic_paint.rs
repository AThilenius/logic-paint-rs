use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    session::SerdeSession,
    viewport::{Msg as YewViewportMsg, Viewport},
};

#[wasm_bindgen]
pub struct LogicPaint {
    handle: AppHandle<Viewport>,
}

#[wasm_bindgen]
impl LogicPaint {
    #[wasm_bindgen(constructor)]
    pub fn new(root: Element, on_edit_callback: &js_sys::Function) -> LogicPaint {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let handle = yew::start_app_in_element::<Viewport>(root);
        handle.send_message(YewViewportMsg::SetOnEditCallback(on_edit_callback.clone()));

        LogicPaint { handle }
    }

    pub fn set_session_from_string(&mut self, data: &str) -> Option<String> {
        let trimmed = data.trim();

        if !trimmed.starts_with("LPS1:") {
            return Some("String is not valid LogicPaint Session Version 1 data.".to_string());
        }

        let compressed_bytes = {
            if let Ok(compressed_bytes) = base64::decode(&trimmed[5..]) {
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
        let component = self.handle.get_component().unwrap_throw();
        let bytes = bincode::serialize(&SerdeSession::from(&component.session)).unwrap_throw();
        let compressed_bytes = compress_to_vec(&bytes, 6);
        format!("LPS1:{}", base64::encode(compressed_bytes))
    }
}
