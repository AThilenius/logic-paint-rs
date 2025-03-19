use crate::substrate::{
    buffer::Buffer,
    compiler::{Atom, CellPart},
    mask::Mask,
};

use super::{Tool, ToolInput, ToolOutput};

#[derive(Default)]
pub struct ToolPaintMetal {
    // The last complete drawing op. The buffer will be reverted to this state is drawing is
    // cancelled.
    checkpoint: Buffer,
    // Drawing is tracked separately from io_state to allow for primary+secondary cancelling
    drawing: bool,
}

impl Tool for ToolPaintMetal {
    fn tool_name(&self) -> &str {
        "paint-metal"
    }

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
        if io_state.get_key_code("KeyW").clicked {
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
        // from painting, so we handle it separately. These of we. The proverbial we. It's just me.
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

        ToolOutput {
            buffer: Some(buffer),
            mask: Some(mask),
            ..Default::default()
        }
    }
}
