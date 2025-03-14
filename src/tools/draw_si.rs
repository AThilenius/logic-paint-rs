use crate::{
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart},
        mask::Mask,
    },
    upc::{NormalizedCell, Silicon},
};

use super::{Tool, ToolInput, ToolOutput};

#[derive(Default)]
pub struct ToolPaintSi {
    // The last complete drawing op. The buffer will be reverted to this state is drawing is
    // cancelled.
    checkpoint: Buffer,
    // Drawing is tracked separately to io_state to allow for primary+secondary cancelling
    drawing: bool,
}

impl Tool for ToolPaintSi {
    fn activate(&mut self, buffer: Buffer) -> ToolOutput {
        self.drawing = false;
        self.checkpoint = buffer;
        ToolOutput {
            cursor_style: Some("crosshair".to_string()),
            ..Default::default()
        }
    }

    fn deactivate(&mut self, _buffer: Buffer) -> ToolOutput {
        ToolOutput {
            buffer: if self.drawing {
                Some(self.checkpoint.clone())
            } else {
                None
            },
            mask: Some(Default::default()),
            cursor_style: Some("default".to_string()),
            ..Default::default()
        }
    }

    fn dispatch_event(
        &mut self,
        ToolInput {
            active,
            io_state,
            buffer: previous_buffer,
            ..
        }: &ToolInput,
    ) -> ToolOutput {
        if io_state.get_key_code("KeyQ").clicked {
            return ToolOutput {
                take_active: true,
                ..Default::default()
            };
        }

        // Drawing tools have no actions while inactive.
        if !active {
            return Default::default();
        }

        // Cancelling draw (with other mouse button)
        if self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = false;
            return ToolOutput {
                buffer: Some(self.checkpoint.clone()),
                mask: Some(Default::default()),
                ..Default::default()
            };
        }

        // End drawing with commit
        if self.drawing && (io_state.primary.released || io_state.secondary.released) {
            self.drawing = false;
            self.checkpoint = previous_buffer.clone();
            return ToolOutput {
                checkpoint: true,
                ..Default::default()
            };
        }

        let mut buffer = self.checkpoint.clone();

        // Start drawing
        if !self.drawing && (io_state.primary.clicked || io_state.secondary.clicked) {
            self.drawing = true;
        }

        if !self.drawing {
            // We aren't drawing. Peace out, Homie.
            return Default::default();
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

        ToolOutput {
            buffer: Some(buffer),
            mask: Some(mask),
            ..Default::default()
        }
    }
}

// #[derive(Serialize, Deserialize, Default)]
// pub struct ToolVisual {
//     /// The buffer following the mouse, drawn to the `front_buffer`. Not serialized as persisting
//     /// it makes little sense.
//     #[serde(skip)]
//     mouse_follow_buffer: Option<Buffer>,
//
//     /// Map of all saved register buffers.
//     /// TODO
//     #[serde(skip)]
//     registers: HashMap<String, Buffer>,
// }
//
// impl Tool for ToolVisual {
//     fn dispatch_event(
//         &mut self,
//         ToolInput {
//             mut buffer,
//             io_state,
//             editor,
//             ..
//         }: ToolInput,
//     ) -> ToolOutput {
//         USE CURSOR: "cell"
//         // Check if a named register was clicked (we use this in multiple places).
//         let named_register_clicked = "1234567890*"
//             .chars()
//             .map(|c| c.to_string())
//             .filter(|c| io_state.get_key(c).clicked)
//             .next();
//
//         if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.take() {
//             // Handle placing the mouse follow buffer.
//             if io_state.primary.clicked {
//                 buffer.paste_at(io_state.cell, &mouse_follow_buffer);
//             }
//
//             // Right click (and ESC) clears the mouse follow buffer.
//             if io_state.secondary.clicked {
//                 self.mouse_follow_buffer = None;
//             }
//
//             // KeyR will rotate the mouse-follow buffer
//             if io_state.get_key_code("KeyR").clicked {
//                 self.mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
//             }
//
//             // KeyM will mirror the mouse-follow buffer
//             if io_state.get_key_code("KeyM").clicked {
//                 self.mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
//             }
//
//             // Hitting KeyS + any of the named register keys will save the mouse-follow
//             // buffer into the named register.
//             if io_state.get_key_code("KeyS").down {
//                 if let Some(named_register) = &named_register_clicked {
//                     // If it's the clipboard register, also set the clipboard.
//                     if named_register == "*" {
//                         spawn_local(async move {
//                             // let window = web_sys::window().expect("window");
//                             // let nav = window.navigator().clipboard();
//                             // match nav {
//                             //     Some(a) => {
//                             //         let p = a.write_text("please god work");
//                             //         let result = wasm_bindgen_futures::JsFuture::from(p)
//                             //             .await
//                             //             .expect("clipboard populated");
//                             //         log!("clippyboy worked");
//                             //     }
//                             //     None => {
//                             //         warn!("failed to copy clippyboy");
//                             //     }
//                             // };
//                         });
//                     } else {
//                         self.registers
//                             .insert(named_register.clone(), mouse_follow_buffer.clone());
//                     }
//                     editor.selection = Default::default();
//                 }
//             } else {
//                 // Otherwise override the mouse-follow buffer with the newly selected
//                 // register, if it exists.
//                 if let Some(named_register) = &named_register_clicked {
//                     if let Some(buffer) = self.registers.get(named_register) {
//                         self.mouse_follow_buffer = Some(buffer.clone());
//                     }
//                 }
//             }
//         } else {
//             if io_state.primary.down {
//                 if let Some(drag) = io_state.drag {
//                     editor.selection =
//                         Selection::from_rectangle_inclusive(drag.start, io_state.cell);
//                 }
//             } else if io_state.secondary.down {
//                 editor.selection = Default::default();
//             }
//
//             // Delete selection
//             if io_state.get_key_code("KeyD").clicked {
//                 if !io_state.get_key("Shift").down {
//                     self.mouse_follow_buffer =
//                         Some(buffer.clone_selection(&editor.selection, io_state.cell));
//                 }
//                 buffer.clear_selection(&editor.selection);
//                 editor.selection = Default::default();
//             }
//
//             // Yank selection to mouse-follow buffer
//             if io_state.get_key_code("KeyY").clicked {
//                 self.mouse_follow_buffer =
//                     Some(buffer.clone_selection(&editor.selection, io_state.cell));
//                 editor.selection = Default::default();
//             }
//
//             // Hitting KeyS + any of the named register keys will save the selected cells
//             // into the named register.
//             if io_state.get_key_code("KeyS").down && !editor.selection.is_zero() {
//                 if let Some(named_register) = &named_register_clicked {
//                     let buffer = buffer.clone_selection(&editor.selection, io_state.cell);
//
//                     // If it's the clipboard register, also set the clipboard.
//                     if named_register == "*" {
//                         spawn_local(async move {
//                             let window = web_sys::window().expect("window");
//                             // let nav = window.navigator().clipboard();
//                             // match nav {
//                             //     Some(a) => {
//                             //         let p = a.write_text("please god work");
//                             //         let result = wasm_bindgen_futures::JsFuture::from(p)
//                             //             .await
//                             //             .expect("clipboard populated");
//                             //         log!("clippyboy worked");
//                             //     }
//                             //     None => {
//                             //         warn!("failed to copy clippyboy");
//                             //     }
//                             // };
//                         });
//                     } else {
//                         self.registers.insert(named_register.clone(), buffer);
//                     }
//                     editor.selection = Default::default();
//                 }
//             } else {
//                 // Hitting any of the named register keys (while not holding KeyS) will load
//                 // the register into the mouse-follow buffer.
//                 if let Some(named_register) = named_register_clicked {
//                     // If it's the clipboard register then we have to request the clipboard
//                     // from JS and wait for it to come back. Sucks.
//                     if named_register == "*" {
//                         // notify_js_request_clipboard();
//                         // TODO: Need to REQUEST, not set
//                         spawn_local(async move {
//                             // let window = web_sys::window().expect("window");
//                             // let nav = window.navigator().clipboard();
//                             // match nav {
//                             //     Some(a) => {
//                             //         let p = a.write_text("please god work");
//                             //         let result = wasm_bindgen_futures::JsFuture::from(p)
//                             //             .await
//                             //             .expect("clipboard populated");
//                             //         log!("clippyboy worked");
//                             //     }
//                             //     None => {
//                             //         warn!("failed to copy clippyboy");
//                             //     }
//                             // };
//                         });
//                     } else if let Some(buffer) = self.registers.get(&named_register) {
//                         self.mouse_follow_buffer = Some(buffer.clone());
//                     }
//                     editor.selection = Default::default();
//                 }
//             }
//         }
//
//         // If the mouse follow buffer is set after dispatch, render it to the buffer
//         if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
//             buffer.paste_at(io_state.cell, mouse_follow_buffer)
//         }
//
//         return ToolOutput::Continue {
//             buffer,
//             mask: Default::default(),
//         };
//     }
// }
