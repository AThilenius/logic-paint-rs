use crate::{
    coords::CellCoord,
    module::Module,
    substrate::{buffer::Buffer, execution_context::ExecutionContext, io::IoState, mask::Mask},
    tools::{
        camera_controller::ToolCameraController, draw_metal::ToolPaintMetal, draw_si::ToolPaintSi,
        place_socket::ToolPlaceSocket, visual::ToolVisual, Tool, ToolInput, ToolOutput,
    },
    utils::Selection,
    wgl2::Camera,
};
use wasm_bindgen::prelude::*;

/// Contains all the underlying state of a project, including the active buffer, mask, modules,
/// history and tools.
#[wasm_bindgen(getter_with_clone)]
pub struct Project {
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

    /// Modules
    modules: Vec<Box<dyn Module>>,

    /// All loaded tools
    tools: Vec<Box<dyn Tool>>,

    /// The active tool
    active_tool: usize,

    /// When set, buffer must be static.
    execution_context: Option<ExecutionContext>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct EditorDispatchResult {
    pub buffer_persist: Option<Buffer>,
    pub tools_persist: Vec<ToolPersist>,
    pub camera: Option<Camera>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ToolPersist {
    pub tool_name: String,
    pub serialized_state: Vec<u8>,
}

#[wasm_bindgen]
impl Project {
    #[wasm_bindgen(constructor)]
    pub fn new(buffer: Buffer) -> Self {
        let mut tools = vec![];

        // Create and activate visual as the default tool
        let mut visual = ToolVisual::default();
        let tool_output = visual.activate(buffer.clone());

        // Store tools
        tools.push(Box::new(visual) as Box<dyn Tool>);
        tools.push(Box::new(ToolPaintSi::default()) as Box<dyn Tool>);
        tools.push(Box::new(ToolPaintMetal::default()) as Box<dyn Tool>);
        tools.push(Box::new(ToolPlaceSocket::default()) as Box<dyn Tool>);
        tools.push(Box::new(ToolCameraController::default()) as Box<dyn Tool>);

        Self {
            buffer,
            mask: Default::default(),
            selection: Default::default(),
            cursor_coord: None,
            cursor_style: tool_output
                .cursor_style
                .unwrap_or_else(|| "default".to_string()),
            modules: vec![],
            tools,
            active_tool: 0,
            execution_context: None,
        }
    }

    pub fn dispatch_event(&mut self, io_state: &IoState, camera: &Camera) -> EditorDispatchResult {
        self.cursor_coord = Some(io_state.cell);

        let mut new_active = None;
        let mut dispatch_result = EditorDispatchResult {
            buffer_persist: None,
            tools_persist: vec![],
            camera: None,
        };

        let mut tool_input = ToolInput {
            active: false,
            io_state: io_state.clone(),
            camera: camera.clone(),
            buffer: self.buffer.clone(),
            selection: self.selection,
        };

        let a = self.active_tool;
        let owned_output: Vec<_> = self
            .tools
            .iter_mut()
            .enumerate()
            .map(|(idx, tool)| {
                tool_input.active = idx == a;
                let output = tool.dispatch_event(&tool_input);
                (idx, output)
            })
            .collect();

        for (idx, output) in owned_output {
            if output.take_active && new_active.is_none() && idx != self.active_tool {
                new_active = Some(idx);
            }

            self.handle_dispatch_result(&mut dispatch_result, idx, output);
        }

        if let Some(new_active) = new_active {
            tool_input.active = false;
            let output = self.tools[self.active_tool].deactivate(self.buffer.clone());

            self.selection = Default::default();

            self.handle_dispatch_result(&mut dispatch_result, self.active_tool, output);

            tool_input.active = true;
            self.active_tool = new_active;
            let output = self.tools[self.active_tool].activate(self.buffer.clone());

            self.handle_dispatch_result(&mut dispatch_result, self.active_tool, output);
        }

        dispatch_result
    }

    fn handle_dispatch_result(
        &mut self,
        dispatch_result: &mut EditorDispatchResult,
        idx: usize,
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

        if let Some(selection) = output.selection {
            self.selection = selection;
        }

        if let Some(camera) = output.camera {
            dispatch_result.camera = Some(camera);
        }

        if output.checkpoint {
            dispatch_result.buffer_persist = Some(self.buffer.clone());
        }

        if let Some(bytes) = output.persist_tool_state {
            dispatch_result.tools_persist.push(ToolPersist {
                tool_name: self.tools[idx].get_name().to_string(),
                serialized_state: bytes,
            });
        }
    }
}
