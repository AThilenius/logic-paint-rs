use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Element;
use yew::prelude::*;

use crate::viewport::{
    blueprint::Blueprint, editor_state::SerdeEditorState, Msg as YewViewportMsg, Viewport,
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
        on_editor_state_callback: &js_sys::Function,
        request_clipboard: &js_sys::Function,
        set_clipboard: &js_sys::Function,
    ) -> LogicPaint {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let handle = yew::start_app_in_element::<Viewport>(root);
        handle.send_message(YewViewportMsg::SetJsCallbacks {
            on_edit_callback: on_edit_callback.clone(),
            on_editor_state_callback: on_editor_state_callback.clone(),
            request_clipboard: request_clipboard.clone(),
            set_clipboard: set_clipboard.clone(),
        });

        LogicPaint { handle }
    }

    pub fn set_clipboard(&mut self, data: &str) -> Option<String> {
        self.handle
            .send_message(YewViewportMsg::SetClipboard(data.to_owned()));
        None
    }

    pub fn set_blueprint_from_json_string(&mut self, data: &str) -> Option<String> {
        if let Ok(blueprint) = serde_json::from_str::<Blueprint>(data) {
            self.handle
                .send_message(YewViewportMsg::SetBlueprint(blueprint));
            None
        } else {
            Some(format!(
                "Failed to deserialize JSON, or structure is invalid: {}",
                data
            ))
        }
    }

    pub fn set_editor_state_from_json_string(&mut self, data: &str) -> Option<String> {
        if let Ok(serde_editor_state) = serde_json::from_str::<SerdeEditorState>(data) {
            self.handle
                .send_message(YewViewportMsg::SetEditorState(serde_editor_state.into()));
            None
        } else {
            Some("Failed to deserialize JSON, or structure is invalid.".to_owned())
        }
    }

    pub fn get_editor_state(&self) -> String {
        let component = self
            .handle
            .get_component()
            .expect("Failed to get Yew component");
        serde_json::to_string_pretty(&SerdeEditorState::from(&component.editor_state))
            .expect("Failed to serialize SerdeEditorState")
    }
}
