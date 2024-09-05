use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::tools::ToolPlugin;

/// Editor is home for all the transient editing context, like Tools (and which is selected),
/// number buffers, and so on. It does not "own" any Viewports, but is instead the visitor that
/// will 'draw to' a Viewport. Visually, the Editor wraps around all viewports while editing. There
/// is also only one of them.
#[derive(Default)]
#[wasm_bindgen]
pub struct Editor {
    tools: HashMap<String, ToolPlugin>,
}

#[wasm_bindgen]
impl Editor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mount_tool(&mut self, tool_id: String, tool: ToolPlugin) {
        self.tools.insert(tool_id, tool);
    }
}
