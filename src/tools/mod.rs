use wasm_bindgen::prelude::*;

use crate::substrate::{buffer::Buffer, input::InputState};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ToolPlugin {
    mount(): void;
    unmount(): void;
    dispatch(
        input_state: InputState,
        buffer: Buffer,
    ): Buffer | undefined;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ToolPlugin")]
    pub type ToolPlugin;

    #[wasm_bindgen(method)]
    pub fn mount(this: &ToolPlugin);

    #[wasm_bindgen(method)]
    pub fn unmount(this: &ToolPlugin);

    #[wasm_bindgen(method)]
    pub fn dispatch(this: &ToolPlugin, input_state: InputState, buffer: Buffer) -> Option<Buffer>;
}

// A WASM provided Tool, wired up in JS
#[wasm_bindgen]
pub struct WasmTool;

#[wasm_bindgen]
impl WasmTool {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmTool
    }

    pub fn mount(&mut self) {}

    pub fn unmount(&mut self) {}

    pub fn dispatch(&mut self, _input_state: InputState, mut buffer: Buffer) -> Option<Buffer> {
        buffer.draw_si((0, 0).into(), (-5, -5).into(), true, true);
        Some(buffer)
    }
}
