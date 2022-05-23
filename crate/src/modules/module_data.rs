use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

use crate::{
    coords::CellCoord,
    modules::{MemoryData, RegisterData, TogglePinData},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ModuleData {
    Memory(Rc<RefCell<MemoryData>>),
    Register(Rc<RefCell<RegisterData>>),
    TogglePin(Rc<RefCell<TogglePinData>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    /// The cell coordinate where this pin lives.
    pub coord: CellCoord,

    /// Set to true if the module itself is driving the pin. Modules internally set this value,
    /// and it is subsequently read by the ExecutionContext during simulation.
    pub output_high: bool,
    // Note that the concept of `input_high` is unnecessary, as each individual module can chose to
    // track or ignore that information during the `set_input_pins` callback.
}

impl Pin {
    pub fn new(coord: IVec2) -> Self {
        Self {
            coord: CellCoord(coord),
            output_high: false,
        }
    }

    pub fn new_repeating(start: IVec2, offset: IVec2, n: usize) -> Vec<Pin> {
        let mut cursor = start + offset;
        let mut pins = Vec::new();
        for _ in 0..n {
            pins.push(Pin::new(cursor));
            cursor += offset;
        }

        pins
    }
}

impl ModuleData {
    pub fn get_anchor(&self) -> Anchor {
        match self {
            ModuleData::Memory(m) => m.borrow_mut().get_anchor(),
            ModuleData::Register(m) => m.borrow_mut().get_anchor(),
            ModuleData::TogglePin(m) => m.borrow().get_anchor(),
        }
    }

    pub fn get_pins(&self) -> Vec<Pin> {
        match self {
            ModuleData::Memory(m) => m.borrow_mut().get_pins(),
            ModuleData::Register(m) => m.borrow_mut().get_pins(),
            ModuleData::TogglePin(m) => m.borrow().get_pins(),
        }
    }

    pub fn set_input_pins(&mut self, states: &Vec<bool>) {
        match self {
            ModuleData::Memory(m) => m.borrow_mut().set_input_pins(states),
            ModuleData::Register(m) => m.borrow_mut().set_input_pins(states),
            ModuleData::TogglePin(m) => m.borrow_mut().set_input_pins(states),
        }
    }

    pub fn update(&mut self, _time: f64) {
        match self {
            _default => {}
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
