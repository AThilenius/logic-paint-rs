use std::{cell::RefCell, rc::Rc};

use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::Html;

use crate::coords::{CellCoord, CellCoordOffset};

mod clock;
mod const_value;
mod memory;
mod module_serde;
mod register;
mod toggle_pin;

pub use clock::*;
pub use const_value::*;
pub use memory::*;
pub use module_serde::*;
pub use register::*;
pub use toggle_pin::*;

#[derive(Clone)]
pub struct RootedModule {
    pub root: CellCoord,
    pub module: Rc<RefCell<dyn Module>>,
    pub html: Html,
    pub module_serde: ModuleSerde,
}

pub trait Module {
    fn get_pins(&self) -> Vec<Pin>;

    fn set_pins(&mut self, _pins: &Vec<Pin>) {}

    fn clock(&mut self, _time: f64) {}
}

impl RootedModule {
    pub fn get_pin_coords(&self) -> Vec<CellCoord> {
        self.module
            .borrow()
            .get_pins()
            .iter()
            .map(|p| p.coord_offset.to_cell_coord(self.root))
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    pub label: String,
    pub right_align: bool,

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
    pub fn new(x: i32, y: i32, output_high: bool, label: &str, right_align: bool) -> Self {
        Self {
            label: label.to_owned(),
            right_align,
            coord_offset: CellCoordOffset((x, y).into()),
            output_high,
            input_high: false,
        }
    }

    pub fn new_repeating(
        start: IVec2,
        offset: IVec2,
        n: usize,
        label_prefix: &str,
        right_align: bool,
    ) -> Vec<Pin> {
        let mut cursor = start;
        let mut pins = Vec::new();
        for i in 0..n {
            pins.push(Pin::new(
                cursor.x,
                cursor.y,
                false,
                &format!("{}{}", label_prefix, i),
                right_align,
            ));
            cursor += offset;
        }

        pins
    }
}
