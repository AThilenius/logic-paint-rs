use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::coords::CellCoord;

// mod clock;
mod pin;
mod value;

// pub use clock::*;
pub use pin::*;
pub use value::*;

#[derive(Clone, Serialize, Deserialize)]
#[enum_dispatch(Module)]
pub enum ConcreteModule {
    // Clock(Clock),
    Value(Value),
}

#[enum_dispatch]
pub trait Module {
    fn get_root(&self) -> CellCoord;
    fn set_root(&mut self, root: CellCoord);
    fn get_pins(&self) -> Vec<Pin>;
    fn set_pins(&mut self, _pins: &Vec<Pin>) {}
    fn clock(&mut self, _time: f64) {}
}

impl ConcreteModule {
    pub fn get_pin_coords(&self) -> Vec<CellCoord> {
        self.get_pins()
            .iter()
            .map(|p| p.coord_offset.to_cell_coord(self.get_root()))
            .collect()
    }

    pub fn set_pin_states(&mut self, states: &Vec<bool>) {
        let mut pins = self.get_pins().clone();
        for i in 0..states.len() {
            pins[i].input_high = states[i];
        }
        self.set_pins(&pins);
    }
}
