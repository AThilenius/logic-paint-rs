use bevy::math::IVec2;

use crate::{canvas::Cell, sim::Network};

#[derive(Default)]
pub struct Canvas {
    // NOTE: Mutated in-place, and copied to CanvasHistory when a snapshot is committed, or during
    // an un-do operation.
    pub cells: im::HashMap<IVec2, Cell>,
}

pub struct CanvasHistory {
    pub snapshots: Vec<Canvas>,
}

impl Canvas {
    pub fn compile_to_network(&self) -> Network {
        Network::compile_canvas(self)
    }
}
