pub mod buffer;
pub mod buffer_brush;
pub mod buffer_serde;
pub mod compiler;
mod compress;
pub mod execution_context;
pub mod input;
pub mod label_builder;
pub mod mask;

// use crate::{
//     dom::{DomIntervalHooks, RawInput},
//     substrate::{
//         buffer::Buffer,
//         compiler::{Atom, CellPart},
//         execution_context::ExecutionContext,
//         input::InputState,
//         label_builder::LabelBuilder,
//     },
//     upc::{NormalizedCell, Silicon},
//     utils::Selection,
//     wgl2::{Camera, RenderContext},
// };
//
// use self::mask::Mask;
//
// pub struct Viewport {
//     pub selection: Selection,
//     pub camera: Camera,
//     pub registers: HashMap<String, Buffer>,
//     pub active_buffer: Buffer,
//     pub ephemeral_buffer: Option<Buffer>,
//     pub time: f64,
//     pub input_state: InputState,
//     mouse_follow_buffer: Option<Buffer>,
//     mode: Mode,
//     canvas: NodeRef,
//     render_context: Option<RenderContext>,
//     dom_events: Option<DomIntervalHooks>,
//     on_edit_callback: Option<js_sys::Function>,
//     request_clipboard: Option<js_sys::Function>,
//     set_clipboard: Option<js_sys::Function>,
//     event_hooks: Vec<EventListener>,
// }
//
// pub enum Msg {
//     None,
//     RawInput(RawInput),
//     Render(f64),
//     SetFocus(bool),
// }
//
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
//     /// (E) Enters execution mode
//     /// (R): Run (for now just one clock per frame)
//     /// (C): Enter manual-mode, clocks once.
//     /// (T): Enter manual-mode, ticks once.
//     /// (P): Enter manual-mode
//     Execute(Execution),
//
//     /// (Enter) Starts Label mode.
//     /// (ESC, Enter, Primary, Secondary) Leaves label model.
//     Label(LabelBuilder),
// }
//
// pub struct Execution {
//     pub manual: bool,
//     pub context: ExecutionContext,
// }
//
// impl Viewport {
//     fn draw(&mut self, time: f64) {
//         self.time = time;
//         let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
//
//         // Handle execution.
//         if let Mode::Execute(execution) = &mut self.mode {
//             if !execution.manual {
//                 execution.context.clock_once();
//             }
//             execution.context.update_buffer_mask();
//         }
//
//         // Maintain HTML Canvas size and context viewport.
//         let w = canvas.client_width() as u32;
//         let h = canvas.client_height() as u32;
//
//         if w != canvas.width() || h != canvas.height() {
//             canvas.set_width(w);
//             canvas.set_height(h);
//         }
//
//         let size = Vec2::new(w as f32, h as f32);
//         self.camera.update(size);
//
//         // Redraw the mouse-follow buffer to the ephemeral buffer each frame.
//         if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
//             let mut buffer = self.active_buffer.clone();
//             buffer.paste_at(self.input_state.cell, mouse_follow_buffer);
//             self.ephemeral_buffer = Some(buffer);
//         }
//
//         if let Some(render_context) = &mut self.render_context {
//             let buffer = self
//                 .ephemeral_buffer
//                 .as_ref()
//                 .unwrap_or(&self.active_buffer);
//
//             let highlight_mask = {
//                 match &self.mode {
//                     Mode::PaintSi(Some(atom)) | Mode::PaintMetallic(Some(atom)) => {
//                         let mask = Mask::from_highlight_trace(
//                             self.ephemeral_buffer
//                                 .as_ref()
//                                 .unwrap_or(&self.active_buffer),
//                             *atom,
//                         );
//                         Some(mask)
//                     }
//                     _ => None,
//                 }
//             };
//
//             render_context
//                 .draw(
//                     time,
//                     buffer,
//                     &self.selection,
//                     match (&highlight_mask, &self.mode) {
//                         (Some(highlight_mask), _) => Some(highlight_mask),
//                         (_, Mode::Execute(execution)) => Some(&execution.context.buffer_mask),
//                         _ => None,
//                     },
//                     &self.camera,
//                 )
//                 .expect("Failed to draw render context");
//         }
//     }
//
//     fn dispatch_input_state(&mut self) {
//         // Handle cursor-follow before anything else.
//         if let Some(render_context) = &self.render_context {
//             render_context.set_cursor_coord(self.input_state.cell);
//         }
//
//         // Let the camera take all events beyond that. However, we need to suppress space when in
//         // label mode.
//         if !(matches!(self.mode, Mode::Label(_))
//             && self.input_state.key_codes_down.contains("Space"))
//         {
//             if self.camera.handle_input(&self.input_state) {
//                 // Let JS know the camera changed.
//                 // if let Some(editor_state_callback) = &self.on_editor_state_callback {
//                 //     let json =
//                 //         serde_json::to_string_pretty(&SerdeEditorState::from(&self.editor_state))
//                 //             .expect("Failed to serialize SerdeEditorState");
//                 //     let js_str = JsValue::from(&json);
//                 //     let _ = editor_state_callback.call1(&JsValue::null(), &js_str);
//                 // }
//
//                 // Then early return.
//                 return;
//             }
//         }
//
//         // Check if a named register was clicked (we use this in multiple places).
//         let named_register_clicked = "1234567890*"
//             .chars()
//             .map(|c| c.to_string())
//             .filter(|c| self.input_state.key_clicked == *c)
//             .next();
//
//         // Escape is a global keybind, it always brings us back to Visual mode
//         if self.input_state.key_code_clicked == "Escape" {
//             self.mode = Mode::Visual;
//             self.selection = Default::default();
//             self.ephemeral_buffer = None;
//             self.mouse_follow_buffer = None;
//         }
//
//         // The rest of the keybinds only make sense when not typing a label.
//         if !matches!(self.mode, Mode::Label(..)) {
//             // Enter => Label, Esc => Visual, D => PaintSi, F => PaintMetallic
//             if self.input_state.key_code_clicked == "Enter" {
//                 self.mode = Mode::Label(LabelBuilder::default());
//                 self.selection = Default::default();
//                 self.ephemeral_buffer = None;
//
//                 // Return so that we don't send the initial enter to the builder
//                 return;
//             }
//
//             if self.input_state.key_code_clicked == "KeyQ" {
//                 self.mode = Mode::PaintSi(None);
//                 self.selection = Default::default();
//                 self.mouse_follow_buffer = None;
//             } else if self.input_state.key_code_clicked == "KeyW" {
//                 self.mode = Mode::PaintMetallic(None);
//                 self.selection = Default::default();
//                 self.mouse_follow_buffer = None;
//             } else if self.input_state.key_code_clicked == "KeyE"
//                 && !matches!(self.mode, Mode::Execute(..))
//             {
//                 self.mode = Mode::Execute(Execution {
//                     manual: true,
//                     context: ExecutionContext::compile_from_buffer(&self.active_buffer),
//                 });
//                 self.selection = Default::default();
//                 self.mouse_follow_buffer = None;
//             }
//         }
//
//         let mut set_mouse_follow_buffer = false;
//         let mut new_mouse_follow_buffer = None;
//         let mut notify_js = false;
//
//         match &mut self.mode {
//             Mode::Visual => {
//                 // TODO: Get rid of this clone call.
//                 if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
//                     // Handle placing the mouse follow buffer.
//                     if self.input_state.primary_clicked {
//                         self.active_buffer
//                             .paste_at(self.input_state.cell, &mouse_follow_buffer);
//
//                         notify_js = true;
//                     }
//
//                     // Right click (and ESC) clears the mouse follow buffer.
//                     if self.input_state.secondary {
//                         set_mouse_follow_buffer = true;
//                         new_mouse_follow_buffer = None;
//                         self.ephemeral_buffer = None;
//                     }
//
//                     // KeyR will rotate the mouse-follow buffer
//                     if self.input_state.key_code_clicked == "KeyR" {
//                         set_mouse_follow_buffer = true;
//                         new_mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
//                     }
//
//                     // KeyM will mirror the mouse-follow buffer
//                     if self.input_state.key_code_clicked == "KeyM" {
//                         set_mouse_follow_buffer = true;
//                         new_mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
//                     }
//
//                     // Hitting KeyS + any of the named register keys will save the mouse-follow
//                     // buffer into the named register.
//                     if self.input_state.key_codes_down.contains("KeyS") {
//                         if let Some(named_register) = &named_register_clicked {
//                             // If it's the clipboard register, also set the clipboard.
//                             if named_register == "*" {
//                                 self.notify_js_set_clipboard(&mouse_follow_buffer);
//                             } else {
//                                 self.registers
//                                     .insert(named_register.clone(), mouse_follow_buffer.clone());
//                             }
//                             self.selection = Default::default();
//                         }
//                     } else {
//                         // Otherwise override the mouse-follow buffer with the newly selected
//                         // register, if it exists.
//                         if let Some(named_register) = &named_register_clicked {
//                             if let Some(buffer) = self.registers.get(named_register) {
//                                 set_mouse_follow_buffer = true;
//                                 new_mouse_follow_buffer = Some(buffer.clone());
//                             }
//                         }
//                     }
//                 } else {
//                     if self.input_state.primary {
//                         if let Some(drag) = self.input_state.drag {
//                             self.selection = Selection::from_rectangle_inclusive(
//                                 drag.start,
//                                 self.input_state.cell,
//                             );
//                         }
//                     } else if self.input_state.secondary {
//                         self.selection = Default::default();
//                     }
//
//                     // Delete selection
//                     if self.input_state.key_code_clicked == "KeyD" {
//                         if !self.input_state.shift {
//                             set_mouse_follow_buffer = true;
//                             new_mouse_follow_buffer = Some(
//                                 self.active_buffer
//                                     .clone_selection(&self.selection, self.input_state.cell),
//                             );
//                         }
//                         self.active_buffer.clear_both_in_selection(&self.selection);
//                         self.selection = Default::default();
//                         notify_js = true;
//                     }
//
//                     // Yank selection to mouse-follow buffer
//                     if self.input_state.key_code_clicked == "KeyY" {
//                         set_mouse_follow_buffer = true;
//                         new_mouse_follow_buffer = Some(
//                             self.active_buffer
//                                 .clone_selection(&self.selection, self.input_state.cell),
//                         );
//                         self.selection = Default::default();
//                     }
//
//                     // Hitting KeyS + any of the named register keys will save the selected cells
//                     // into the named register.
//                     if self.input_state.key_codes_down.contains("KeyS") && !self.selection.is_zero()
//                     {
//                         if let Some(named_register) = &named_register_clicked {
//                             let buffer = self
//                                 .active_buffer
//                                 .clone_selection(&self.selection, self.input_state.cell);
//
//                             // If it's the clipboard register, also set the clipboard.
//                             if named_register == "*" {
//                                 self.notify_js_set_clipboard(&buffer);
//                             } else {
//                                 self.registers.insert(named_register.clone(), buffer);
//                             }
//                             self.selection = Default::default();
//                         }
//                     } else {
//                         // Hitting any of the named register keys (while not holding KeyS) will load
//                         // the register into the mouse-follow buffer.
//                         if let Some(named_register) = named_register_clicked {
//                             // If it's the clipboard register then we have to request the clipboard
//                             // from JS and wait for it to come back. Sucks.
//                             if named_register == "*" {
//                                 self.notify_js_request_clipboard();
//                             } else if let Some(buffer) = self.registers.get(&named_register) {
//                                 set_mouse_follow_buffer = true;
//                                 new_mouse_follow_buffer = Some(buffer.clone());
//                             }
//                             self.selection = Default::default();
//                         }
//                     }
//                 }
//             }
//             Mode::PaintMetallic(_) | Mode::PaintSi(_) => {
//                 self.dispatch_paint_input_state();
//             }
//             Mode::Execute(Execution { manual, context }) => {
//                 if self.input_state.key_code_clicked == "KeyR" {
//                     *manual = false;
//                 } else if self.input_state.key_code_clicked == "KeyC" {
//                     *manual = true;
//                     context.clock_once();
//                 } else if self.input_state.key_code_clicked == "KeyT" {
//                     *manual = true;
//                     context.tick_once();
//                 } else if self.input_state.key_code_clicked == "KeyP" {
//                     *manual = true;
//                 }
//             }
//             Mode::Label(label_builder) => {
//                 label_builder.dispatch_input(&self.input_state);
//                 set_mouse_follow_buffer = true;
//                 new_mouse_follow_buffer = Some(label_builder.render_to_buffer(true));
//
//                 // Handle placing the text.
//                 if self.input_state.primary_clicked {
//                     self.active_buffer.paste_at(
//                         self.input_state.cell,
//                         &label_builder.render_to_buffer(false),
//                     );
//
//                     notify_js = true;
//                 }
//             }
//         }
//
//         if set_mouse_follow_buffer {
//             self.mouse_follow_buffer = new_mouse_follow_buffer;
//         }
//
//         if notify_js {
//             self.notify_js_on_change();
//         }
//
//         if self.input_state.key_code_clicked == "KeyF" {
//             self.active_buffer.fix_all_cells();
//         }
//     }
//
//     fn dispatch_paint_input_state(&mut self) {
//         // If neither button is clicked
//         if !self.input_state.primary && !self.input_state.secondary {
//             // Commit the ephemeral buffer if we have one.
//             if let Some(buffer) = self.ephemeral_buffer.take() {
//                 self.active_buffer = buffer;
//
//                 // Notify JS.
//                 self.notify_js_on_change();
//             }
//
//             return;
//         }
//
//         // Painting generates a totally new Buffer (cloned from active) each time.
//         let mut buffer = self.active_buffer.clone();
//
//         let path = self.input_state.get_impulse_drag_path();
//
//         // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
//         // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
//         if self.input_state.ctrl {
//             match self.mode {
//                 Mode::PaintMetallic(_) => path.into_iter().for_each(|c| buffer.clear_metal(c)),
//                 Mode::PaintSi(_) => path.into_iter().for_each(|c| buffer.clear_si(c)),
//                 _ => {}
//             }
//         } else {
//             // Input modes are much, much more complicated. That logic is delegated to it's own file
//             // because they are so stupid-complicated.
//             let mut from = None;
//
//             for cell_coord in &path {
//                 match self.mode {
//                     Mode::PaintMetallic(_) => {
//                         // Primary paints metal, secondary places a Via (only once).
//                         if self.input_state.primary {
//                             buffer.draw_metal(from, *cell_coord);
//                         } else if self.input_state.secondary {
//                             buffer.draw_via(from, *cell_coord);
//                         }
//                     }
//                     Mode::PaintSi(_) => {
//                         // Primary paints N-type, secondary paints P-type.
//                         if self.input_state.primary {
//                             buffer.draw_si(from, *cell_coord, true);
//                         } else {
//                             buffer.draw_si(from, *cell_coord, false);
//                         }
//                     }
//                     _ => {}
//                 }
//                 from = Some(*cell_coord);
//             }
//
//             // Handle highlighting the trace as you draw.
//             match &mut self.mode {
//                 Mode::PaintMetallic(atom) => {
//                     *atom = path.first().map(|c| Atom {
//                         coord: *c,
//                         part: CellPart::Metal,
//                     });
//                 }
//                 Mode::PaintSi(atom) => {
//                     *atom = None;
//                     if path.len() > 0 {
//                         let first = path[0];
//                         let first_cell = NormalizedCell::from(buffer.get_cell(path[0]));
//
//                         if let Silicon::NP { .. } = first_cell.si {
//                             *atom = Some(Atom {
//                                 coord: first,
//                                 part: CellPart::Si,
//                             });
//                         } else if path.len() > 1 {
//                             let second = path[1];
//                             let ec_up_left = first.0.x > second.0.x || first.0.y < second.0.y;
//                             *atom = Some(Atom {
//                                 coord: first,
//                                 part: if ec_up_left {
//                                     CellPart::EcUpLeft
//                                 } else {
//                                     CellPart::EcDownRight
//                                 },
//                             });
//                         }
//                     }
//                 }
//                 _ => {}
//             }
//         }
//
//         self.ephemeral_buffer = Some(buffer);
//     }
//
//     fn notify_js_on_change(&self) {
//         if let Some(on_edit_callback) = self.on_edit_callback.as_ref() {
//             let json = serde_json::to_string_pretty(&self.active_buffer)
//                 .expect("Failed to serialize Blueprint");
//             let js_str = JsValue::from(&json);
//             let _ = on_edit_callback.call1(&JsValue::null(), &js_str);
//         }
//     }
//
//     fn notify_js_request_clipboard(&self) {
//         if let Some(request_clipboard) = self.request_clipboard.as_ref() {
//             let _ = request_clipboard.call0(&JsValue::null());
//         }
//     }
//
//     fn notify_js_set_clipboard(&self, buffer: &Buffer) {
//         if let Some(set_clipboard) = self.set_clipboard.as_ref() {
//             let json =
//                 serde_json::to_string_pretty(&buffer).expect("Failed to serialize Blueprint");
//             let js_str = JsValue::from(&json);
//             let _ = set_clipboard.call1(&JsValue::null(), &js_str);
//         }
//     }
// }
