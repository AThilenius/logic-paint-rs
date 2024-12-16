use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::{
    coords::CellCoord,
    log,
    substrate::{buffer::Buffer, io::IoState, mask::Mask},
    tools::{Tool, ToolInput, ToolOutput, ToolPaintMetal},
    utils::Selection,
    wgl2::Camera,
};

/// An Editor represents the underlying 'state' of an edit session, including the buffer data,
/// transient buffers, masks, tools, and active tool states. It can be thought of as an active
/// 'file'. It does not include anything having to do with the presentation of the editor, like
/// cameras, viewports, and so on.
#[wasm_bindgen(getter_with_clone)]
pub struct Editor {
    /// The active buffer that dispatched input will be rendered to (like drawing).
    /// This is used as the base for rendering (with mouse-follow stacked on top of it).
    pub buffer: Buffer,

    /// The current render mask applied to the buffer.
    pub mask: Mask,

    /// The selected (visual mode) cells
    pub selection: Selection,

    /// The last used cursor location
    pub cursor_coord: Option<CellCoord>,

    /// The CSS style that should be applied to the cursor.
    pub cursor_style: String,

    /// The current mode of the standard editor.
    /// mode: Mode,
    tools: HashMap<String, Box<dyn Tool>>,

    active_tool: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct EditorDispatchResult {
    pub buffer_persist: Option<Buffer>,
    pub tools_persist: Vec<ToolPersist>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ToolPersist {
    pub tool_name: String,
    pub serialized_state: Vec<u8>,
}

#[wasm_bindgen]
impl Editor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            buffer: Default::default(),
            mask: Default::default(),
            selection: Default::default(),
            cursor_coord: None,
            cursor_style: "default".to_string(),
            tools: HashMap::from([
                // (
                //     "visual".to_string(),
                //     Box::new(ToolVisual::default()) as Box<dyn Tool>,
                // ),
                // (
                //     "paint-si".to_string(),
                //     Box::new(ToolPaintSi::default()) as Box<dyn Tool>,
                // ),
                (
                    "paint-metal".to_string(),
                    Box::new(ToolPaintMetal::default()) as Box<dyn Tool>,
                ),
            ]),
            active_tool: "none".to_string(),
        }
    }

    pub fn dispatch_event(&mut self, io_state: &IoState, camera: &Camera) -> EditorDispatchResult {
        self.cursor_coord = Some(io_state.cell);

        let active_tool = self.active_tool.clone();
        let mut new_active = None;
        let mut dispatch_result = EditorDispatchResult {
            buffer_persist: None,
            tools_persist: vec![],
        };

        let mut tool_input = ToolInput {
            active: false,
            io_state: io_state.clone(),
            camera: camera.clone(),
            buffer: self.buffer.clone(),
            selection: self.selection,
        };

        let owned_output: Vec<_> = self
            .tools
            .iter_mut()
            .map(|(name, tool)| {
                tool_input.active = *name == active_tool;
                let output = tool.dispatch_event(&tool_input);
                (name.to_string(), output)
            })
            .collect();

        for (name, output) in owned_output {
            if output.take_active && !tool_input.active && new_active.is_none() {
                new_active = Some(name.to_string());
            }

            self.handle_dispatch_result(&mut dispatch_result, name, output);
        }

        if let Some(new_active) = new_active {
            if self.active_tool != "none" {
                tool_input.active = false;
                let output = self
                    .tools
                    .get_mut(&self.active_tool)
                    .unwrap()
                    .deactivate(&tool_input);

                self.handle_dispatch_result(
                    &mut dispatch_result,
                    self.active_tool.to_string(),
                    output,
                );
            }

            tool_input.active = true;
            self.active_tool = new_active;
            let output = self
                .tools
                .get_mut(&self.active_tool)
                .unwrap()
                .activate(&tool_input);

            self.handle_dispatch_result(&mut dispatch_result, self.active_tool.to_string(), output);
        }

        dispatch_result
    }

    fn handle_dispatch_result(
        &mut self,
        dispatch_result: &mut EditorDispatchResult,
        name: String,
        output: ToolOutput,
    ) {
        if let Some(buffer) = output.buffer {
            self.buffer = buffer;
        }

        if let Some(mask) = output.mask {
            self.mask = mask;
        }

        if let Some(cursor_style) = output.cursor_style {
            self.cursor_style = cursor_style;
        }

        if output.checkpoint {
            dispatch_result.buffer_persist = Some(self.buffer.clone());
        }

        if let Some(bytes) = output.persist_tool_state {
            dispatch_result.tools_persist.push(ToolPersist {
                tool_name: name,
                serialized_state: bytes,
            });
        }
    }
}
