use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::{
    substrate::{buffer::Buffer, input::InputState},
    viewport::Viewport,
};

// Define the trait with bounds on Serialize and DeserializeOwned
pub trait Serializable: Serialize + for<'de> Deserialize<'de> {}

// Automatically implement the Serializable trait for all types
// that already implement Serialize and DeserializeOwned
impl<T> Serializable for T where T: Serialize + for<'de> Deserialize<'de> {}

pub trait Tool: Serializable {
    fn dispatch_input(
        &mut self,
        viewport: &mut Viewport,
        buffer: &mut Buffer,
        input_state: InputState,
    ) -> bool;
}

#[derive(Serialize, Deserialize)]
pub struct ToolPaintMetal {}

impl Tool for ToolPaintMetal {
    fn dispatch_input(
        &mut self,
        viewport: &mut Viewport,
        buffer: &mut Buffer,
        input_state: InputState,
    ) -> bool {
        // If neither button is clicked
        if !input_state.primary && !input_state.secondary {
            viewport.buffer = buffer.clone();
            return true;
        }

        if let Some(drag) = input_state.drag {
            buffer.draw_metal(drag.start, input_state.cell, drag.initial_impulse_vertical);
        } else {
            buffer.draw_metal(input_state.cell, input_state.cell, false);
        }

        true
    }
}

// pub struct VisualTool {
//     selection: Selection,
//     mouse_follow_buffer: Option<Buffer>,
//     registers: HashMap<String, Buffer>,
// }
//
// impl Tool for VisualTool {
//     fn dispatch_input(
//         &mut self,
//         viewport: &mut Viewport,
//         buffer: &mut Buffer,
//         input_state: InputState,
//     ) -> bool {
//         // Check if a named register was clicked (we use this in multiple places).
//         let named_register_clicked = "1234567890*"
//             .chars()
//             .map(|c| c.to_string())
//             .filter(|c| input_state.key_clicked == *c)
//             .next();
//
//         if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.clone() {
//             // Handle placing the mouse follow buffer.
//             if input_state.primary_clicked {
//                 buffer.paste_at(input_state.cell, &mouse_follow_buffer);
//             }
//
//             // Right click (and ESC) clears the mouse follow buffer.
//             if input_state.secondary {
//                 self.mouse_follow_buffer = None;
//             }
//
//             // KeyR will rotate the mouse-follow buffer
//             if input_state.key_code_clicked == "KeyR" {
//                 self.mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
//             }
//
//             // KeyM will mirror the mouse-follow buffer
//             if input_state.key_code_clicked == "KeyM" {
//                 self.mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
//             }
//
//             // Hitting KeyS + any of the named register keys will save the mouse-follow
//             // buffer into the named register.
//             if input_state.key_codes_down.contains("KeyS") {
//                 if let Some(named_register) = &named_register_clicked {
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
//                         self.registers
//                             .insert(named_register.clone(), mouse_follow_buffer.clone());
//                     }
//                     self.selection = Default::default();
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
//             if input_state.primary {
//                 if let Some(drag) = input_state.drag {
//                     self.selection =
//                         Selection::from_rectangle_inclusive(drag.start, input_state.cell);
//                 }
//             } else if input_state.secondary {
//                 self.selection = Default::default();
//             }
//
//             // Delete selection
//             if input_state.key_code_clicked == "KeyD" {
//                 if !input_state.shift {
//                     self.mouse_follow_buffer =
//                         Some(buffer.clone_selection(&self.selection, input_state.cell));
//                 }
//                 buffer.clear_selection(&self.selection);
//                 self.selection = Default::default();
//             }
//
//             // Yank selection to mouse-follow buffer
//             if input_state.key_code_clicked == "KeyY" {
//                 self.mouse_follow_buffer =
//                     Some(buffer.clone_selection(&self.selection, input_state.cell));
//                 self.selection = Default::default();
//             }
//
//             // Hitting KeyS + any of the named register keys will save the selected cells
//             // into the named register.
//             if input_state.key_codes_down.contains("KeyS") && !self.selection.is_zero() {
//                 if let Some(named_register) = &named_register_clicked {
//                     let buffer = self
//                         .completed_buffer
//                         .clone_selection(&self.selection, input_state.cell);
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
//                     self.selection = Default::default();
//                 }
//             } else {
//                 // Hitting any of the named register keys (while not holding KeyS) will load
//                 // the register into the mouse-follow buffer.
//                 if let Some(named_register) = named_register_clicked {
//                     // If it's the clipboard register then we have to request the clipboard
//                     // from JS and wait for it to come back. Sucks.
//                     if named_register == "*" {
//                         // self.notify_js_request_clipboard();
//                         // TODO: Need to REQUEST, not set
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
//                     } else if let Some(buffer) = self.registers.get(&named_register) {
//                         self.mouse_follow_buffer = Some(buffer.clone());
//                     }
//                     self.selection = Default::default();
//                 }
//             }
//         }
//
//         false
//     }
// }
