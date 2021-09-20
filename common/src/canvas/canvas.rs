use std::collections::HashMap;

use bevy::math::IVec2;

use crate::{
    canvas::{Cell, DEFAULT_CANVAS_SIZE},
    sim::Network,
    utils::{HilbertArray, HilbertIndexing},
};

use super::{Metal, Silicon};

/// Component that represents and entire mutable Metal Oxide Silicon canvas.
pub struct Canvas {
    pub size: usize,
    pub io_pins: HashMap<IVec2, IOPin>,
    pub cells: HilbertArray<Cell>,
}

impl Canvas {
    pub fn compile_to_network(&self) -> Network {
        Network::compile_canvas(self)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        let mut s = Self {
            size: DEFAULT_CANVAS_SIZE,
            io_pins: HashMap::new(),
            cells: HilbertArray::new_2d(DEFAULT_CANVAS_SIZE),
        };

        // Dev
        let io_loc = IVec2::new(
            (DEFAULT_CANVAS_SIZE / 2) as i32,
            (DEFAULT_CANVAS_SIZE / 2) as i32,
        );
        s.io_pins.insert(
            io_loc,
            IOPin {
                loc: io_loc,
                is_constant: true,
            },
        );
        *s.cells.get_mut(io_loc) = Cell {
            si: Silicon::None,
            metal: Metal::IO {
                dirs: Default::default(),
            },
        };

        s
    }
}

#[derive(Debug)]
pub struct IOPin {
    pub loc: IVec2,
    pub is_constant: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Conductor {
    pub loc: IVec2,
    pub level: ConductorLevel,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ConductorLevel {
    Si,
    Gate,
    Metal,
}
