use glam::IVec2;

use crate::sim::Simulation;

pub trait PinModule {
    /// Returns an array of pin definitions. Pin states will be provided in the same order during
    /// update.
    fn get_pin_definitions(&mut self) -> Vec<PinDefinition>;

    /// Takes in a Simulation reference, as well as an array of pin states (given in the same order
    /// that `get_pin_definitions` returned). Returns a same-length array of pins this module wishes
    /// to externally drive (not pins that are already driven internally).
    fn update(&mut self, sim: &Simulation, pin_states: Vec<bool>) -> Vec<bool>;
}

pub struct PinDefinition {
    pub name: String,
}

pub struct MountedPinModule {
    pub module: Box<dyn PinModule>,
    pub mounted_cell_locs: Vec<IVec2>,
}
