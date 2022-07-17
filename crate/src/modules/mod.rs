use std::{cell::RefCell, rc::Rc};

use yew::Html;

use crate::coords::CellCoord;

mod clock;
mod memory;
mod module_serde;
mod pin;
mod register;
mod toggle_pin;
mod value;

pub use clock::*;
pub use memory::*;
pub use module_serde::*;
pub use pin::*;
pub use register::*;
pub use toggle_pin::*;
pub use value::*;

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
