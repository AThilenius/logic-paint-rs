use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    blueprint::Blueprint,
    viewport::{Msg as YewViewportMsg, Viewport},
};

#[wasm_bindgen]
pub struct LogicPaint {
    handle: AppHandle<Viewport>,
}

#[wasm_bindgen]
impl LogicPaint {
    #[wasm_bindgen(constructor)]
    pub fn new(
        root: Element,
        on_edit_callback: &js_sys::Function,
        request_clipboard: &js_sys::Function,
        set_clipboard: &js_sys::Function,
    ) -> LogicPaint {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let handle = yew::start_app_in_element::<Viewport>(root);
        handle.send_message(YewViewportMsg::SetJsCallbacks {
            on_edit_callback: on_edit_callback.clone(),
            request_clipboard: request_clipboard.clone(),
            set_clipboard: set_clipboard.clone(),
        });

        LogicPaint { handle }
    }

    pub fn set_clipboard(&mut self, data: &str) -> Option<String> {
        if let Ok(blueprint) = serde_json::from_str::<Blueprint>(data) {
            self.handle
                .send_message(YewViewportMsg::SetClipboard(blueprint));
            None
        } else {
            Some("Failed to deserialize JSON, or structure is invalid.".to_owned())
        }
    }

    pub fn set_partial_blueprint_from_json_string(&mut self, data: &str) -> Option<String> {
        if let Ok(blueprint) = serde_json::from_str::<Blueprint>(data) {
            self.handle
                .send_message(YewViewportMsg::SetBlueprintPartial(blueprint));
            None
        } else {
            Some("Failed to deserialize JSON, or structure is invalid.".to_owned())
        }
    }

    pub fn get_blueprint_as_json_string(&mut self) -> String {
        let component = self.handle.get_component().unwrap_throw();
        serde_json::to_string_pretty(&Blueprint::from(&component.active_buffer)).unwrap_throw()
    }
}
