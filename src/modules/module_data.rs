use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

use crate::{coords::CellCoord, modules::TogglePinData};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ModuleData {
    TogglePin(Rc<RefCell<TogglePinData>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    /// The cell coordinate where this pin lives.
    pub coord: CellCoord,

    /// Set to true if the circuit it driving the pin.
    pub input_high: bool,

    /// Set to true if the module itself is driving the pin.
    pub output_high: bool,
}

impl ModuleData {
    pub fn reset(&mut self) {
        match self {
            ModuleData::TogglePin(m) => m.borrow_mut().reset(),
        }
    }

    pub fn get_anchor(&self) -> Anchor {
        match self {
            ModuleData::TogglePin(m) => m.borrow().get_anchor(),
        }
    }

    pub fn get_pins(&self) -> Vec<Pin> {
        match self {
            ModuleData::TogglePin(m) => m.borrow().get_pins(),
        }
    }

    pub fn set_input_pins(&mut self, states: &Vec<bool>) {
        match self {
            ModuleData::TogglePin(m) => m.borrow_mut().set_input_pins(states),
        }
    }

    pub fn update(&mut self, time: f64) {
        match self {
            default => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub root: CellCoord,
    pub align: Alignment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}
