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
    fn get_name(&self) -> &str {
        "paint-si"
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
