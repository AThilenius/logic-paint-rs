use js_sys::JsString;
use wasm_bindgen::prelude::*;

use crate::{coords::CellCoord, error};

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pin {
    // Where this pin is set
    pub cell_coord: CellCoord,

    // If changes in `si_output_high` should trigger the socket update callback. Multiple pin
    // triggers will not cause the socket to be updated more than once per clock.
    pub trigger: bool,

    // Set to true when the substrate is driving the pin high.
    pub si_output_high: bool,

    // Set to true by the module, to drive the substrate line high.
    pub si_input_high: bool,
}

#[wasm_bindgen]
#[derive(Clone, PartialEq, Eq)]
pub struct Socket {
    // If the socket should be updated every clock or not.
    pub always_update: bool,

    // The pins this socket owns.
    #[wasm_bindgen(skip)]
    pub pins: Vec<Pin>,

    // Set to true when a pin trigger is changed.
    pub(crate) pending_update: bool,

    // The callback invoked each clock cycle if `always_update` is set, or a trigger pin changes
    // output states.
    update_callback: js_sys::Function,
}

#[wasm_bindgen]
impl Pin {
    #[wasm_bindgen(constructor)]
    pub fn new(cell_coord: CellCoord, trigger: bool) -> Self {
        Self {
            cell_coord,
            trigger,
            si_output_high: false,
            si_input_high: false,
        }
    }
}

#[wasm_bindgen]
impl Socket {
    #[wasm_bindgen(constructor)]
    pub fn new(pins: Vec<Pin>, always_update: bool, update_callback: js_sys::Function) -> Self {
        Self {
            pins,
            always_update,
            update_callback,
            // Update at least once on first cycle.
            pending_update: true,
        }
    }

    pub(crate) fn invoke_update_callback(&mut self) {
        if !self.pending_update || self.always_update {
            return;
        }

        self.pending_update = false;

        let output_states = js_sys::Array::new();
        for pin in &self.pins {
            output_states.push(&js_sys::Boolean::from(pin.si_output_high));
        }

        match self.update_callback.call1(&JsValue::NULL, &output_states) {
            Ok(res) => {
                // Ignore anything that isn't an array
                if !res.is_array() {
                    return;
                }

                let res = js_sys::Array::from(&res);

                for i in 0..res.length() {
                    self.pins[i as usize].si_input_high = res
                        .get(i)
                        .dyn_into::<js_sys::Boolean>()
                        .unwrap_or_default()
                        .value_of();
                }
            }
            Err(err) => {
                error!(
                    "Failed to invoke socket callback: {}",
                    JsString::from(err).to_string()
                );
            }
        }
    }
}
