use std::{cell::RefCell, rc::Rc};

use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::Html;

use crate::coords::{CellCoord, CellCoordOffset};

mod clock;
mod memory;
mod module_serde;
mod register;
mod toggle_pin;

pub use clock::*;
pub use memory::*;
pub use module_serde::*;
pub use register::*;
pub use toggle_pin::*;

#[derive(Clone)]
pub struct AnchoredModule {
    pub anchor: Anchor,
    pub module: Rc<RefCell<dyn Module>>,
    pub html: Html,
    pub module_serde: ModuleSerde,
}

pub trait Module {
    fn get_pins(&self) -> Vec<Pin>;

    fn set_pins(&mut self, _pins: &Vec<Pin>) {}

    fn tick(&mut self, _time: f64) {}
}

impl AnchoredModule {
    pub fn get_pint_coords(&self) -> Vec<CellCoord> {
        self.module
            .borrow()
            .get_pins()
            .iter()
            .map(|p| p.coord_offset.to_cell_coord(self.anchor.root))
            .collect()
    }

    pub fn set_pin_states(&mut self, states: &Vec<bool>) {
        let mut pins = self.module.borrow().get_pins().clone();
        for i in 0..states.len() {
            pins[i].input_high = states[i];
        }
        self.module.borrow_mut().set_pins(&pins);
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    /// The cell offset coordinate where this pin lives, relative to it's anchor.
    pub coord_offset: CellCoordOffset,

    /// Set to true if the module itself is driving the pin. Modules internally set this value,
    /// and it is subsequently read by the ExecutionContext during simulation.
    pub output_high: bool,

    /// Set to true if the module is being driven by a substrate trace. The execution context sets
    /// this value, and it is subsequently read by the module.
    pub input_high: bool,
}

impl Pin {
    pub fn new(x: i32, y: i32, output_high: bool) -> Self {
        Self {
            coord_offset: CellCoordOffset((x, y).into()),
            output_high,
            input_high: false,
        }
    }

    pub fn new_repeating(start: IVec2, offset: IVec2, n: usize) -> Vec<Pin> {
        let mut cursor = start;
        let mut pins = Vec::new();
        for _ in 0..n {
            pins.push(Pin::new(cursor.x, cursor.y, false));
            cursor += offset;
        }

        pins
    }
}
