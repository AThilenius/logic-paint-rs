use std::collections::HashMap;

use crate::{
    log,
    substrate::buffer::{Buffer, COUNT},
    utils::Selection,
};

use super::{Tool, ToolInput, ToolOutput};

#[derive(Default)]
pub struct ToolVisual {
    // The last complete drawing op. The buffer will be reverted to this state is drawing is
    // cancelled.
    checkpoint: Buffer,

    /// The buffer following the mouse, drawn to the `front_buffer`. Not serialized as persisting
    /// it makes little sense.
    mouse_follow_buffer: Option<Buffer>,

    /// The selected cells. Persisted until tool is deactivated.
    selection: Selection,

    /// Map of all saved register buffers.
    /// TODO serialize
    registers: HashMap<String, Buffer>,
}

impl Tool for ToolVisual {
    fn activate(&mut self, buffer: Buffer) -> ToolOutput {
        self.checkpoint = buffer;
        ToolOutput {
            cursor_style: Some("cell".to_string()),
            ..Default::default()
        }
    }

    fn deactivate(&mut self, _buffer: Buffer) -> ToolOutput {
        self.selection = Default::default();
        self.mouse_follow_buffer = None;
        ToolOutput {
            buffer: Some(self.checkpoint.clone()),
            mask: Some(Default::default()),
            cursor_style: Some("default".to_string()),
            ..Default::default()
        }
    }

    fn dispatch_event(
        &mut self,
        ToolInput {
            active, io_state, ..
        }: &ToolInput,
    ) -> ToolOutput {
        let start = *COUNT.lock().unwrap().borrow();

        if io_state.get_key_code("Escape").clicked {
            self.selection = Default::default();
            self.mouse_follow_buffer = None;
            return ToolOutput {
                take_active: true,
                ..Default::default()
            };
        }

        if !active {
            return Default::default();
        }

        let mut buffer = self.checkpoint.clone();
        let mut checkpoint = false;

        // Check if a named register was clicked (we use this in multiple places).
        let named_register_clicked = "1234567890*"
            .chars()
            .map(|c| c.to_string())
            .filter(|c| io_state.get_key(c).clicked)
            .next();

        if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.clone() {
            // Handle placing the mouse follow buffer.
            if io_state.primary.clicked {
                self.checkpoint
                    .paste_at(io_state.cell, &mouse_follow_buffer);
                checkpoint = true;
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
                        // TODO: Escalate to clipboard.
                    } else {
                        self.registers
                            .insert(named_register.clone(), mouse_follow_buffer.clone());
                    }
                    self.selection = Default::default();
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
                    self.selection = Selection::from_rectangle_inclusive(drag.start, io_state.cell);
                }
            } else if io_state.secondary.down {
                self.selection = Default::default();
            }

            // Delete selection
            if io_state.get_key_code("KeyD").clicked {
                if !io_state.get_key("Shift").down {
                    self.mouse_follow_buffer =
                        Some(buffer.clone_selection(&self.selection, io_state.cell));
                }
                buffer.clear_selection(&self.selection);
                self.selection = Default::default();
            }

            // Yank selection to mouse-follow buffer
            if io_state.get_key_code("KeyY").clicked {
                self.mouse_follow_buffer =
                    Some(buffer.clone_selection(&self.selection, io_state.cell));
                self.selection = Default::default();
            }

            // Hitting KeyS + any of the named register keys will save the selected cells
            // into the named register.
            if io_state.get_key_code("KeyS").down && !self.selection.is_zero() {
                if let Some(named_register) = &named_register_clicked {
                    let buffer = buffer.clone_selection(&self.selection, io_state.cell);

                    // If it's the clipboard register, also set the clipboard.
                    self.registers.insert(named_register.clone(), buffer);
                    // TODO: escalate
                    self.selection = Default::default();
                }
            } else {
                // Hitting any of the named register keys (while not holding KeyS) will load
                // the register into the mouse-follow buffer.
                if let Some(named_register) = named_register_clicked {
                    // If it's the clipboard register then we have to request the clipboard
                    // from JS and wait for it to come back. Sucks.
                    if named_register == "*" {
                        // TODO: escalate
                    } else if let Some(buffer) = self.registers.get(&named_register) {
                        self.mouse_follow_buffer = Some(buffer.clone());
                    }
                    self.selection = Default::default();
                }
            }
        }

        // If the mouse follow buffer is set after dispatch, render it to the buffer
        if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
            buffer.paste_at(io_state.cell, mouse_follow_buffer)
        }

        let end = *COUNT.lock().unwrap().borrow();
        log!("Visual tool called set {} times", end - start);

        ToolOutput {
            buffer: Some(buffer),
            checkpoint,
            selection: Some(self.selection),
            ..Default::default()
        }
    }
}
