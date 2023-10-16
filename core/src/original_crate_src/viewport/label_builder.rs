use glam::IVec2;

use crate::{
    coords::CellCoord,
    log,
    utils::Selection,
    viewport::{blueprint::Blueprint, buffer::Buffer, input::InputState},
};

use super::brush;

pub struct LabelBuilder {
    text: String,
    font_face_buffer: Buffer,
    cursor: usize,
}

impl Default for LabelBuilder {
    fn default() -> Self {
        let font_face_buffer = {
            if let Ok(bp) =
                serde_json::from_str::<Blueprint>(include_str!("../../templates/font_file.lpbp"))
            {
                bp.into()
            } else {
                log!("Failed to deserialize JSON, or structure is invalid.");
                Buffer::default()
            }
        };

        Self {
            text: Default::default(),
            font_face_buffer,
            cursor: 0,
        }
    }
}

impl LabelBuilder {
    pub fn dispatch_input(&mut self, input_state: &InputState) {
        // A bit of a hack: check if the key is 'printable'.
        if input_state.key_clicked.len() == 1 {
            self.text
                .insert(self.cursor, input_state.key_clicked.chars().nth(0).unwrap());
            self.cursor += 1;
        } else {
            match input_state.key_code_clicked.as_str() {
                "Enter" => {
                    self.text.insert(self.cursor, '\n');
                    self.cursor += 1;
                }
                "Backspace" => {
                    if self.cursor > 0 {
                        self.text.remove(self.cursor - 1);
                        self.cursor -= 1;

                        // Continue removing whole word if Ctrl was held.
                        if input_state.ctrl {
                            while self.cursor > 0
                                && self.text.chars().nth(self.cursor - 1).unwrap() != ' '
                            {
                                self.text.remove(self.cursor - 1);
                                self.cursor -= 1;
                            }
                        }
                    }
                }
                "ArrowLeft" => self.cursor = if self.cursor > 0 { self.cursor - 1 } else { 0 },
                "ArrowRight" => {
                    self.cursor = if self.cursor < self.text.len() {
                        self.cursor + 1
                    } else {
                        self.text.len()
                    }
                }
                _ => {}
            }
        }
    }

    pub fn render_to_buffer(&self, render_markers: bool) -> Buffer {
        let mut buffer = Buffer::default();
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        for c in self.text.chars() {
            if c == '\n' {
                cursor_y -= 4;
                cursor_x = 0;
                continue;
            }

            if !c.is_ascii() {
                continue;
            }

            let ascii = c as u8;

            // Space starts at 32. Everything before that are control signals.
            let index = (ascii as i32) - 32;
            let ll = IVec2::new(index * 3, 0);

            let character_buffer = self.font_face_buffer.clone_selection(
                &Selection {
                    lower_left: CellCoord(ll),
                    upper_right: CellCoord(ll + IVec2::new(3, 3)),
                },
                CellCoord(ll),
            );

            buffer.paste_at(CellCoord(IVec2::new(cursor_x, cursor_y)), &character_buffer);
            cursor_x += 3;
        }

        // Draw cursor
        if render_markers {
            let mut c_x = 0;
            let mut c_y = 0;

            for (i, c) in self.text.chars().enumerate() {
                if i >= self.cursor {
                    break;
                }

                if c == '\n' {
                    c_y -= 4;
                    c_x = 0;
                } else {
                    c_x += 3;
                }
            }

            c_y -= 1;
            brush::draw_metal(&mut buffer, None, CellCoord(IVec2::new(c_x, c_y)));
            for _ in 1..5 {
                brush::draw_metal(
                    &mut buffer,
                    Some(CellCoord(IVec2::new(c_x, c_y))),
                    CellCoord(IVec2::new(c_x, c_y + 1)),
                );
                c_y += 1;
            }
        }

        buffer
    }
}
