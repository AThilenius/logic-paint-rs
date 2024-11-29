use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::{
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart},
        io::IoState,
        mask::Mask,
    },
    upc::{NormalizedCell, Silicon},
    utils::Selection,
    viewport::Viewport,
    wgl2::Camera,
};

pub trait Tool {
    fn activate(&mut self, ctx: &mut ToolDispatchCtx) {
        let _ = ctx;
    }

    fn deactivate(&mut self, ctx: &mut ToolDispatchCtx) {
        let _ = ctx;
    }

    fn dispatch_event(&mut self, ctx: ToolDispatchCtx) -> ToolOutput;
}

pub struct ToolDispatchCtx<'a> {
    pub viewport: &'a mut Viewport,
    pub camera: &'a mut Camera,
    pub io_state: &'a mut IoState,
    pub buffer: Buffer,
}

pub enum ToolOutput {
    NoOp,
    Reset,
    Continue { buffer: Buffer, mask: Mask },
    Commit { maintain_mask: bool },
}

#[derive(Serialize, Deserialize, Default)]
pub struct ToolPaintMetal {
    // Drawing is tracked seperately to io_state to allow for primary+secondary cancelling
    drawing: bool,
}

impl Tool for ToolPaintMetal {
    fn dispatch_event(
        &mut self,
        ToolDispatchCtx {
            mut buffer,
            io_state,
            ..
        }: ToolDispatchCtx,
    ) -> ToolOutput {
        // Cancelling draw (with other mouse button)
        if self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = false;
            return ToolOutput::Reset;
        }

        // End drawing with commit
        if self.drawing && (io_state.primary.released || io_state.secondary.released) {
            self.drawing = false;
            return ToolOutput::Commit {
                maintain_mask: true,
            };
        }

        // Start drawing
        if !self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = true;
        }

        if !self.drawing {
            return ToolOutput::NoOp;
        }

        let path = io_state.get_drag_path();
        let mut mask = Mask::default();

        // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
        // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
        if io_state.get_key("Control").down {
            path.into_iter().for_each(|c| buffer.clear_cell_metal(c))
        } else {
            let mut from = None;

            for cell_coord in &path {
                // Primary paints metal, secondary places a Via (only once).
                if io_state.primary.down {
                    buffer.draw_metal_link(from, *cell_coord);
                } else if io_state.secondary.down {
                    buffer.draw_via(*cell_coord);
                }
                from = Some(*cell_coord);
            }

            // Create a highlight mask for the highlighted atom (if any)
            if let Some(&coord) = path.first() {
                mask = Mask::from_highlight_trace(
                    &buffer,
                    Atom {
                        coord,
                        part: CellPart::Metal,
                    },
                );
            }
        }

        ToolOutput::Continue { buffer, mask }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ToolPaintSi {
    // Drawing is tracked seperately to io_state to allow for primary+secondary cancelling
    drawing: bool,
}

impl Tool for ToolPaintSi {
    fn dispatch_event(
        &mut self,
        ToolDispatchCtx {
            mut buffer,
            io_state,
            ..
        }: ToolDispatchCtx,
    ) -> ToolOutput {
        // Cancelling draw (with other mouse button)
        if self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = false;
            return ToolOutput::Reset;
        }

        // End drawing with commit
        if self.drawing && (io_state.primary.released || io_state.secondary.released) {
            self.drawing = false;
            return ToolOutput::Commit {
                maintain_mask: true,
            };
        }

        // Start drawing
        if !self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = true;
        }

        if !self.drawing {
            return ToolOutput::NoOp;
        }

        let path = io_state.get_drag_path();
        let mut mask = Mask::default();

        // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
        // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
        if io_state.get_key("Control").down {
            path.into_iter().for_each(|c| buffer.clear_cell_si(c));
        } else {
            // Input modes are much, much more complicated. That logic is delegated to it's own file
            // because they are so stupid-complicated.
            let mut from = None;

            for cell_coord in &path {
                buffer.draw_si_link(from, *cell_coord, io_state.primary.down);
                from = Some(*cell_coord);
            }

            // Handle highlighting the trace as you draw.
            if path.len() > 0 {
                let first = path[0];
                let first_cell = NormalizedCell::from(buffer.get_cell(path[0]));

                if let Silicon::NP { .. } = first_cell.si {
                    mask = Mask::from_highlight_trace(
                        &buffer,
                        Atom {
                            coord: first,
                            part: CellPart::Si,
                        },
                    )
                } else if path.len() > 1 {
                    let second = path[1];
                    let ec_up_left = first.0.x > second.0.x || first.0.y < second.0.y;
                    mask = Mask::from_highlight_trace(
                        &buffer,
                        Atom {
                            coord: first,
                            part: if ec_up_left {
                                CellPart::EcUpLeft
                            } else {
                                CellPart::EcDownRight
                            },
                        },
                    );
                }
            }
        }

        ToolOutput::Continue { buffer, mask }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ToolVisual {
    /// The buffer following the mouse, drawn to the `front_buffer`. Not serialized as persisting
    /// it makes little sense.
    #[serde(skip)]
    mouse_follow_buffer: Option<Buffer>,

    /// Map of all saved register buffers.
    /// TODO
    #[serde(skip)]
    registers: HashMap<String, Buffer>,
}

impl Tool for ToolVisual {
    fn dispatch_event(
        &mut self,
        ToolDispatchCtx {
            mut buffer,
            io_state,
            viewport,
            ..
        }: ToolDispatchCtx,
    ) -> ToolOutput {
        // Check if a named register was clicked (we use this in multiple places).
        let named_register_clicked = "1234567890*"
            .chars()
            .map(|c| c.to_string())
            .filter(|c| io_state.get_key(c).clicked)
            .next();

        if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.take() {
            // Handle placing the mouse follow buffer.
            if io_state.primary.clicked {
                buffer.paste_at(io_state.cell, &mouse_follow_buffer);
            }

            // Right click (and ESC) clears the mouse follow buffer.
            if io_state.secondary.clicked {
                self.mouse_follow_buffer = None;
            }

            // KeyR will rotate the mouse-follow buffer
            if io_state.get_key_code("KeyR").clicked {
                self.mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
            }

            // KeyM will mirror the mouse-follow buffer
            if io_state.get_key_code("KeyM").clicked {
                self.mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
            }

            // Hitting KeyS + any of the named register keys will save the mouse-follow
            // buffer into the named register.
            if io_state.get_key_code("KeyS").down {
                if let Some(named_register) = &named_register_clicked {
                    // If it's the clipboard register, also set the clipboard.
                    if named_register == "*" {
                        spawn_local(async move {
                            // let window = web_sys::window().expect("window");
                            // let nav = window.navigator().clipboard();
                            // match nav {
                            //     Some(a) => {
                            //         let p = a.write_text("please god work");
                            //         let result = wasm_bindgen_futures::JsFuture::from(p)
                            //             .await
                            //             .expect("clipboard populated");
                            //         log!("clippyboy worked");
                            //     }
                            //     None => {
                            //         warn!("failed to copy clippyboy");
                            //     }
                            // };
                        });
                    } else {
                        self.registers
                            .insert(named_register.clone(), mouse_follow_buffer.clone());
                    }
                    viewport.selection = Default::default();
                }
            } else {
                // Otherwise override the mouse-follow buffer with the newly selected
                // register, if it exists.
                if let Some(named_register) = &named_register_clicked {
                    if let Some(buffer) = self.registers.get(named_register) {
                        self.mouse_follow_buffer = Some(buffer.clone());
                    }
                }
            }
        } else {
            if io_state.primary.down {
                if let Some(drag) = io_state.drag {
                    viewport.selection =
                        Selection::from_rectangle_inclusive(drag.start, io_state.cell);
                }
            } else if io_state.secondary.down {
                viewport.selection = Default::default();
            }

            // Delete selection
            if io_state.get_key_code("KeyD").clicked {
                if !io_state.get_key("Shift").down {
                    self.mouse_follow_buffer =
                        Some(buffer.clone_selection(&viewport.selection, io_state.cell));
                }
                buffer.clear_selection(&viewport.selection);
                viewport.selection = Default::default();
            }

            // Yank selection to mouse-follow buffer
            if io_state.get_key_code("KeyY").clicked {
                self.mouse_follow_buffer =
                    Some(buffer.clone_selection(&viewport.selection, io_state.cell));
                viewport.selection = Default::default();
            }

            // Hitting KeyS + any of the named register keys will save the selected cells
            // into the named register.
            if io_state.get_key_code("KeyS").down && !viewport.selection.is_zero() {
                if let Some(named_register) = &named_register_clicked {
                    let buffer = buffer.clone_selection(&viewport.selection, io_state.cell);

                    // If it's the clipboard register, also set the clipboard.
                    if named_register == "*" {
                        spawn_local(async move {
                            let window = web_sys::window().expect("window");
                            // let nav = window.navigator().clipboard();
                            // match nav {
                            //     Some(a) => {
                            //         let p = a.write_text("please god work");
                            //         let result = wasm_bindgen_futures::JsFuture::from(p)
                            //             .await
                            //             .expect("clipboard populated");
                            //         log!("clippyboy worked");
                            //     }
                            //     None => {
                            //         warn!("failed to copy clippyboy");
                            //     }
                            // };
                        });
                    } else {
                        self.registers.insert(named_register.clone(), buffer);
                    }
                    viewport.selection = Default::default();
                }
            } else {
                // Hitting any of the named register keys (while not holding KeyS) will load
                // the register into the mouse-follow buffer.
                if let Some(named_register) = named_register_clicked {
                    // If it's the clipboard register then we have to request the clipboard
                    // from JS and wait for it to come back. Sucks.
                    if named_register == "*" {
                        // notify_js_request_clipboard();
                        // TODO: Need to REQUEST, not set
                        spawn_local(async move {
                            // let window = web_sys::window().expect("window");
                            // let nav = window.navigator().clipboard();
                            // match nav {
                            //     Some(a) => {
                            //         let p = a.write_text("please god work");
                            //         let result = wasm_bindgen_futures::JsFuture::from(p)
                            //             .await
                            //             .expect("clipboard populated");
                            //         log!("clippyboy worked");
                            //     }
                            //     None => {
                            //         warn!("failed to copy clippyboy");
                            //     }
                            // };
                        });
                    } else if let Some(buffer) = self.registers.get(&named_register) {
                        self.mouse_follow_buffer = Some(buffer.clone());
                    }
                    viewport.selection = Default::default();
                }
            }
        }

        // If the mouse follow buffer is set after dispatch, render it to the buffer
        if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
            buffer.paste_at(io_state.cell, mouse_follow_buffer)
        }

        return ToolOutput::Continue {
            buffer,
            mask: Default::default(),
        };
    }
}
