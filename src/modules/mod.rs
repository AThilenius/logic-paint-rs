mod toggle_pin;

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

pub use toggle_pin::*;

use crate::coords::CellCoord;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ModuleData {
    TogglePin(Rc<RefCell<TogglePinData>>),
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

    pub fn get_pins(&self) -> Vec<CellCoord> {
        match self {
            ModuleData::TogglePin(m) => vec![m.borrow().get_anchor().root],
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
