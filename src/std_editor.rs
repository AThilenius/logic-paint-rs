use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    substrate::{
        buffer::Buffer, compiler::Atom, input::InputState, label_builder::LabelBuilder, mask::Mask,
    },
    utils::Selection,
    viewport::Viewport,
    wgl2::Camera,
};

pub enum Mode {
    /// (ESC) Default starting mode, accessed from any other mode with ESC.
    /// - Denoted by the cell-cursor (Excel style)
    /// - Only mode where module anchors are visible
    /// - Same selection keybinds as Excel. Clicking/Dragging selected a range. Holding Shift adds
    ///   to the selection. Holding Ctrl removes from the selection.
    /// - Hovering a trace highlights the conductive path
    /// - Double-clicking a trace selects the conductive path cells
    /// - VSCode::OnCopy copies the selected cells and modules, with the root being what ever cell
    ///   was last under the mouse at that time.
    /// - VSCode::OnPaste pastes into a 'cursor follow' buffer, next mouse click commits it to
    ///   active
    Visual,

    /// (F) Paints metal and vias.
    /// LMB: paint
    /// RMB || Shift+LMB: Via
    /// Ctrl+... to remove
    PaintMetallic(Option<Atom>),

    /// (D) Paints doped silicon
    /// LMB: paint N
    /// RMB || Shift+LMB paint P
    /// Ctrl+... to erase any type & mosfets
    PaintSi(Option<Atom>),

    /// (Enter) Starts Label mode.
    /// (ESC, Enter, Primary, Secondary) Leaves label model.
    Label(LabelBuilder),
}

#[derive(Default)]
#[wasm_bindgen]
pub struct StdEdtior {
    pub camera: Camera,

    selection: Selection,
    /// Buffer containing the latest fully completed changes.
    completed_buffer: Buffer,

    /// Map of all saved register buffers.
    registers: HashMap<String, Buffer>,

    /// The most recently dispatched input state.
    input_state: InputState,

    /// The buffer that dispatched input will be rendered to (like drawing). This buffer is
    /// swapped out each frame.
    back_buffer: Buffer,

    /// The buffer following the mouse, drawn to the `front_buffer`
    mouse_follow_buffer: Option<Buffer>,

    /// The current mode of the standard editor.
    mode: Mode,
}

#[wasm_bindgen]
impl StdEdtior {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn render_to_viewport(&mut self, viewport: &mut Viewport) {
        viewport.buffer = self.back_buffer.clone();

        if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
            viewport
                .buffer
                .paste_at(self.input_state.cell, mouse_follow_buffer)
        }

        viewport.mask = {
            match &self.mode {
                Mode::PaintSi(Some(atom)) | Mode::PaintMetallic(Some(atom)) => {
                    Mask::from_highlight_trace(&self.back_buffer, *atom)
                }
                _ => Default::default(),
            }
        };

        viewport.camera = self.camera.clone();
        viewport.cursor = Some(self.input_state.cell);
    }

    pub fn dispatch_input_state(&mut self) {
        // Check if a named register was clicked (we use this in multiple places).
        let named_register_clicked = "1234567890*"
            .chars()
            .map(|c| c.to_string())
            .filter(|c| self.input_state.key_clicked == *c)
            .next();

        // Escape is a global keybind, it always brings us back to Visual mode
        if self.input_state.key_code_clicked == "Escape" {
            self.mode = Mode::Visual;
            self.selection = Default::default();
            self.back_buffer = self.completed_buffer.clone();
            self.mouse_follow_buffer = None;
        }

        // The rest of the keybinds only make sense when not typing a label.
        if !matches!(self.mode, Mode::Label(..)) {
            // Enter => Label, Esc => Visual, D => PaintSi, F => PaintMetallic
            if self.input_state.key_code_clicked == "Enter" {
                self.mode = Mode::Label(LabelBuilder::default());
                self.selection = Default::default();
                self.back_buffer = self.completed_buffer.clone();

                // Return so that we don't send the initial enter to the builder
                return;
            }

            if self.input_state.key_code_clicked == "KeyQ" {
                self.mode = Mode::PaintSi(None);
                self.selection = Default::default();
                self.mouse_follow_buffer = None;
            } else if self.input_state.key_code_clicked == "KeyW" {
                self.mode = Mode::PaintMetallic(None);
                self.selection = Default::default();
                self.mouse_follow_buffer = None;
            }
        }

        match &mut self.mode {
            Mode::Visual => {
                if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.clone() {
                    // Handle placing the mouse follow buffer.
                    if self.input_state.primary_clicked {
                        self.completed_buffer
                            .paste_at(self.input_state.cell, &mouse_follow_buffer);
                    }

                    // Right click (and ESC) clears the mouse follow buffer.
                    if self.input_state.secondary {
                        self.mouse_follow_buffer = None;
                    }

                    // KeyR will rotate the mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyR" {
                        self.mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
                    }

                    // KeyM will mirror the mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyM" {
                        self.mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
                    }

                    // Hitting KeyS + any of the named register keys will save the mouse-follow
                    // buffer into the named register.
                    if self.input_state.key_codes_down.contains("KeyS") {
                        if let Some(named_register) = &named_register_clicked {
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
                    if self.input_state.primary {
                        if let Some(drag) = self.input_state.drag {
                            self.selection = Selection::from_rectangle_inclusive(
                                drag.start,
                                self.input_state.cell,
                            );
                        }
                    } else if self.input_state.secondary {
                        self.selection = Default::default();
                    }

                    // Delete selection
                    if self.input_state.key_code_clicked == "KeyD" {
                        if !self.input_state.shift {
                            self.mouse_follow_buffer = Some(
                                self.completed_buffer
                                    .clone_selection(&self.selection, self.input_state.cell),
                            );
                        }
                        self.completed_buffer.clear_selection(&self.selection);
                        self.selection = Default::default();
                    }

                    // Yank selection to mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyY" {
                        self.mouse_follow_buffer = Some(
                            self.completed_buffer
                                .clone_selection(&self.selection, self.input_state.cell),
                        );
                        self.selection = Default::default();
                    }

                    // Hitting KeyS + any of the named register keys will save the selected cells
                    // into the named register.
                    if self.input_state.key_codes_down.contains("KeyS") && !self.selection.is_zero()
                    {
                        if let Some(named_register) = &named_register_clicked {
                            let buffer = self
                                .completed_buffer
                                .clone_selection(&self.selection, self.input_state.cell);

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
                            self.selection = Default::default();
                        }
                    } else {
                        // Hitting any of the named register keys (while not holding KeyS) will load
                        // the register into the mouse-follow buffer.
                        if let Some(named_register) = named_register_clicked {
                            // If it's the clipboard register then we have to request the clipboard
                            // from JS and wait for it to come back. Sucks.
                            if named_register == "*" {
                                // self.notify_js_request_clipboard();
                                // TODO: Need to REQUEST, not set
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
                            } else if let Some(buffer) = self.registers.get(&named_register) {
                                self.mouse_follow_buffer = Some(buffer.clone());
                            }
                            self.selection = Default::default();
                        }
                    }
                }
            }
            Mode::PaintMetallic(_) | Mode::PaintSi(_) => {
                self.dispatch_paint_input_state();
            }
            Mode::Label(label_builder) => {
                label_builder.dispatch_input(&self.input_state);
                self.mouse_follow_buffer = Some(label_builder.render_to_buffer(true));

                // Handle placing the text.
                if self.input_state.primary_clicked {
                    self.completed_buffer.paste_at(
                        self.input_state.cell,
                        &label_builder.render_to_buffer(false),
                    );
                }
            }
        }

        if self.input_state.key_code_clicked == "KeyF" {
            self.completed_buffer.fix_all_cells();
        }
    }

    fn dispatch_paint_input_state(&mut self) {
        // // If neither button is clicked
        // if !self.input_state.primary && !self.input_state.secondary {
        //     self.completed_buffer = self.back_buffer.clone();
        //     return;
        // }
        //
        // // Painting generates a totally new Buffer (cloned from active) each time.
        // let mut buffer = self.committed_buffer.clone();
        //
        // let path = self.input_state.get_impulse_drag_path();
        //
        // // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
        // // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
        // if self.input_state.ctrl {
        //     match self.mode {
        //         Mode::PaintMetallic(_) => path.into_iter().for_each(|c| buffer.clear_metal(c)),
        //         Mode::PaintSi(_) => path.into_iter().for_each(|c| buffer.clear_si(c)),
        //         _ => {}
        //     }
        // } else {
        //     // Input modes are much, much more complicated. That logic is delegated to it's own file
        //     // because they are so stupid-complicated.
        //     let mut from = None;
        //
        //     for cell_coord in &path {
        //         match self.mode {
        //             Mode::PaintMetallic(_) => {
        //                 // Primary paints metal, secondary places a Via (only once).
        //                 if self.input_state.primary {
        //                     buffer.draw_metal(from, *cell_coord);
        //                 } else if self.input_state.secondary {
        //                     buffer.draw_via(from, *cell_coord);
        //                 }
        //             }
        //             Mode::PaintSi(_) => {
        //                 // Primary paints N-type, secondary paints P-type.
        //                 if self.input_state.primary {
        //                     buffer.draw_si(from, *cell_coord, true);
        //                 } else {
        //                     buffer.draw_si(from, *cell_coord, false);
        //                 }
        //             }
        //             _ => {}
        //         }
        //         from = Some(*cell_coord);
        //     }
        //
        //     // Handle highlighting the trace as you draw.
        //     match &mut self.mode {
        //         Mode::PaintMetallic(atom) => {
        //             *atom = path.first().map(|c| Atom {
        //                 coord: *c,
        //                 part: CellPart::Metal,
        //             });
        //         }
        //         Mode::PaintSi(atom) => {
        //             *atom = None;
        //             if path.len() > 0 {
        //                 let first = path[0];
        //                 let first_cell = NormalizedCell::from(buffer.get_cell(path[0]));
        //
        //                 if let Silicon::NP { .. } = first_cell.si {
        //                     *atom = Some(Atom {
        //                         coord: first,
        //                         part: CellPart::Si,
        //                     });
        //                 } else if path.len() > 1 {
        //                     let second = path[1];
        //                     let ec_up_left = first.0.x > second.0.x || first.0.y < second.0.y;
        //                     *atom = Some(Atom {
        //                         coord: first,
        //                         part: if ec_up_left {
        //                             CellPart::EcUpLeft
        //                         } else {
        //                             CellPart::EcDownRight
        //                         },
        //                     });
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        //
        // self.ephemeral_buffer = Some(buffer);
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Visual
    }
}
