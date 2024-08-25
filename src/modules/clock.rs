use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    coords::CellCoord,
    modules::{Module, Pin},
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Clock {
    pub root: CellCoord,
    pub start_delay: usize,
    pub devisor: usize,

    #[serde(skip)]
    delay: Option<usize>,

    #[serde(skip)]
    high: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            high: false,
            start_delay: 1,
            delay: None,
            devisor: 1,
        }
    }
}

impl Module for Clock {
    fn get_root(&self) -> CellCoord {
        self.root
    }

    fn set_root(&mut self, root: CellCoord) {
        self.root = root;
    }

    fn get_pins(&self) -> Vec<Pin> {
        vec![Pin::new(0, 0, self.high, "CLK", false)]
    }

    fn clock(&mut self, _time: f64) {
        if self.delay.is_none() {
            self.delay = Some(self.start_delay);
        }

        if let Some(delay) = &mut self.delay {
            if *delay > 0 {
                *delay -= 1;
                return;
            }

            *delay = self.devisor;
            self.high = !self.high;
        }
    }
}
