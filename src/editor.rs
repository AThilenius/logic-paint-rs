use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::{
    log,
    substrate::{buffer::Buffer, io::IoState, mask::Mask},
    tools::{Tool, ToolDispatchCtx, ToolOutput, ToolPaintMetal, ToolPaintSi, ToolVisual},
    viewport::Viewport,
    wgl2::Camera,
};

// pub enum Mode {
//     /// (ESC) Default starting mode, accessed from any other mode with ESC.
//     /// - Denoted by the cell-cursor (Excel style)
//     /// - Only mode where module anchors are visible
//     /// - Same selection keybinds as Excel. Clicking/Dragging selected a range. Holding Shift adds
//     ///   to the selection. Holding Ctrl removes from the selection.
//     /// - Hovering a trace highlights the conductive path
//     /// - Double-clicking a trace selects the conductive path cells
//     /// - VSCode::OnCopy copies the selected cells and modules, with the root being what ever cell
//     ///   was last under the mouse at that time.
//     /// - VSCode::OnPaste pastes into a 'cursor follow' buffer, next mouse click commits it to
//     ///   active
//     Visual,
//
//     /// (F) Paints metal and vias.
//     /// LMB: paint
//     /// RMB || Shift+LMB: Via
//     /// Ctrl+... to remove
//     PaintMetallic(Option<Atom>),
//
//     /// (D) Paints doped silicon
//     /// LMB: paint N
//     /// RMB || Shift+LMB paint P
//     /// Ctrl+... to erase any type & mosfets
//     PaintSi(Option<Atom>),
//
//     /// (Enter) Starts Label mode.
//     /// (ESC, Enter, Primary, Secondary) Leaves label model.
//     Label(LabelBuilder),
// }

#[wasm_bindgen]
pub struct Editor {
    /// Buffer containing the latest fully completed changes. It is not used
    /// for rendering directly.
    completed_buffer: Buffer,

    /// The buffer that dispatched input will be rendered to (like drawing).
    /// This is used as the base for rendering (with mouse-follow stacked on
    /// top of it).
    transient_buffer: Buffer,

    transient_mask: Mask,

    /// The current mode of the standard editor.
    /// mode: Mode,
    tools: HashMap<String, Box<dyn Tool>>,

    active_tool: String,
}

#[wasm_bindgen]
impl Editor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            completed_buffer: Default::default(),
            transient_buffer: Default::default(),
            transient_mask: Default::default(),
            tools: HashMap::from([
                (
                    "visual".to_string(),
                    Box::new(ToolVisual::default()) as Box<dyn Tool>,
                ),
                (
                    "paint-si".to_string(),
                    Box::new(ToolPaintSi::default()) as Box<dyn Tool>,
                ),
                (
                    "paint-metal".to_string(),
                    Box::new(ToolPaintMetal::default()) as Box<dyn Tool>,
                ),
            ]),
            active_tool: "visual".to_string(),
        }
    }

    pub fn render_to_viewport(
        &mut self,
        viewport: &mut Viewport,
        camera: &mut Camera,
    ) -> Result<(), JsValue> {
        viewport.draw(
            camera,
            self.transient_buffer.clone(),
            self.transient_mask.clone(),
        )?;
        Ok(())
    }

    pub fn dispatch_event(
        &mut self,
        io_state: &mut IoState,
        viewport: &mut Viewport,
        camera: &mut Camera,
    ) {
        // Cursor-follow
        viewport.cursor = Some(io_state.cell);

        // CSS mouse cursor logic
        // self.output_state.viewport_mouse_cursor =
        //     if self.input_state.key_codes_down.contains("Space") {
        //         "grabbing"
        //     } else {
        //         match self.mode {
        //             Mode::Visual => "cell",
        //             Mode::PaintMetallic(_) | Mode::PaintSi(_) => "crosshair",
        //             // Mode::Execute(..) | Mode::ModuleEdit(None) => "default",
        //             // Mode::Label(..) | Mode::ModuleEdit(Some(..)) => "copy",
        //             Mode::Label(..) => "copy",
        //         }
        //     }
        //     .to_string();

        if camera.handle_input(&io_state) {
            return;
        }

        // Escape is a global keybind, it always brings us back to Visual mode

        if io_state.get_key_code("Escape").clicked {
            self.active_tool = "visual".to_string();
        }

        if io_state.get_key_code("KeyQ").clicked {
            self.active_tool = "paint-si".to_string();
        } else if io_state.get_key_code("KeyW").clicked {
            self.active_tool = "paint-metal".to_string();
        }

        if io_state.get_key_code("KeyF").clicked {
            self.completed_buffer.fix_all_cells();
        }

        if let Some(tool) = self.tools.get_mut(&self.active_tool) {
            let tool_output = tool.dispatch_event(ToolDispatchCtx {
                viewport,
                buffer: self.completed_buffer.clone(),
                camera,
                io_state,
            });

            match tool_output {
                ToolOutput::NoOp => {}
                ToolOutput::Reset => {
                    self.transient_buffer = self.completed_buffer.clone();
                    self.transient_mask = Default::default();
                }
                ToolOutput::Continue { buffer, mask } => {
                    self.transient_buffer = buffer;
                    self.transient_mask = mask;
                }
                ToolOutput::Commit { maintain_mask } => {
                    self.completed_buffer = self.transient_buffer.clone();
                    // TODO: Save to DB

                    if !maintain_mask {
                        self.transient_mask = Default::default();
                    }
                }
            }
        }
    }
}
