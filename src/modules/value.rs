use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    coords::CellCoord,
    modules::{Module, Pin},
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Value {
    pub root: CellCoord,
    pub bus_width: usize,
    pub value: i64,
    pub spacing: usize,

    #[serde(skip)]
    pub value_in: i64,
}

impl Default for Value {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            bus_width: 1,
            value: 0,
            value_in: 0,
            spacing: 1,
        }
    }
}

impl Module for Value {
    fn get_root(&self) -> CellCoord {
        self.root
    }

    fn set_root(&mut self, root: CellCoord) {
        self.root = root;
    }

    fn get_pins(&self) -> Vec<Pin> {
        let mut pins = Pin::new_repeating(
            (0, 0).into(),
            (0, -(self.spacing as i32)).into(),
            self.bus_width,
            "b",
            false,
        );

        let unsigned = unsafe { std::mem::transmute::<i64, u64>(self.value) };
        for i in 0..self.bus_width {
            pins[i].output_high = (unsigned >> i) & 1 > 0;
        }

        pins
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        let mut unsigned = 0_u64;

        for i in 0..self.bus_width {
            if pins[i].input_high {
                unsigned |= 1 << i;
            }
        }

        self.value_in = unsafe { std::mem::transmute::<u64, i64>(unsigned) };
    }
}
