use bevy::math::IVec2;

use crate::{canvas::Cell, sim::Network};

#[derive(Default)]
pub struct Canvas {
    // NOTE: Mutated in-place, and copied to CanvasHistory when a snapshot is committed, or during
    // an un-do operation.
    pub cells: im::HashMap<IVec2, Cell>,

    /// A list of all timing events for I/O pins. This includes things like clocks.
    pub io_timing_events: Vec<IoTimingEvent>,
}

// All I/Os are checked for edge handlers every edge. A handler is just an Enum value.
// Likewise, a list of TimingEvent are listed in the Canvas
// - TimingEvent::Clock { io_loc: IVec2, cycle_delay: usize },
// - TimingEvent::SetOnce { io_loc: IVec2, cycle_delay: usize },
pub enum IoTimingEvent {
    Clock {
        io_loc: IVec2,
        period: usize,
    },
    SetOnce {
        loc: IVec2,
        value: bool,
        delay: usize,
    },
}

pub struct CanvasHistory {
    pub snapshots: Vec<Canvas>,
}

impl Canvas {
    pub fn compile_to_network(&self) -> Network {
        Network::compile_canvas(self)
    }
}
