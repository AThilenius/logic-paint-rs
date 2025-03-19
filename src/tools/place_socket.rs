use crate::{
    log,
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart},
        mask::Mask,
    },
    upc::Bit,
};

use super::{Tool, ToolInput, ToolOutput};

#[derive(Default)]
pub struct ToolPlaceSocket {
    // The last complete drawing op. The buffer will be reverted to this state is drawing is
    // cancelled.
    checkpoint: Buffer,
}

impl Tool for ToolPlaceSocket {
    fn tool_name(&self) -> &str {
        "place-socket"
    }

    fn activate(&mut self, buffer: Buffer) -> ToolOutput {
        self.checkpoint = buffer;
        ToolOutput {
            cursor_style: Some("crosshair".to_string()),
            ..Default::default()
        }
    }

    fn deactivate(&mut self, _buffer: Buffer) -> ToolOutput {
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
        if io_state.get_key_code("KeyE").clicked {
            return ToolOutput {
                take_active: true,
                ..Default::default()
            };
        }

        // Drawing tools have no actions while inactive.
        if !active {
            return Default::default();
        }

        let mut buffer = self.checkpoint.clone();

        if io_state.get_key("Control").down {
            if Bit::get(buffer.get_cell(io_state.cell), Bit::SOCKET) {
                buffer.set_socket(io_state.cell, None);
            }
        } else {
            if !Bit::get(buffer.get_cell(io_state.cell), Bit::SOCKET) {
                buffer.set_socket(io_state.cell, Some("p1".to_string()));
            }
        }

        if io_state.primary.clicked {
            self.checkpoint = buffer;
            return ToolOutput {
                checkpoint: true,
                ..Default::default()
            };
        }

        // Label all sockets
        for socket in buffer.sockets.clone() {
            buffer.draw_label(socket.cell_coord, &socket.name, None);
        }

        ToolOutput {
            buffer: Some(buffer),
            ..Default::default()
        }
    }
}
