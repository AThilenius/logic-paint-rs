/// # Simulation
/// ## Axioms
/// - Gate state is stored as open/closed (bool). This already encompasses NPN vs PNP logic. Ie.
///   "open" means the gate is conductive irrespective of if it's an NPN or PNP.
/// - Gates need to hold handles to both the Base path, as well as both EC paths for state updates
///   (the last step of a sim loop).
/// - I/O pins need to hold handles to their respective paths.
/// - Path state is stored as high/low (bool).
/// - A single simulation loop keeps two copies of both the path state and gate state, one for
///   immutable iteration through the graph, another for the output
/// ## Sim steps
/// - Update input pin states (pull pins for new values)
/// - Update the paths those gates belong to (possibly combine with the above step)
/// - Using the previous run's gate state, propagate the high signals from all high pins through the
///   graph breadth-first. The edge set only expands if a change was made, ie if the gate has
///   mismatched high states (left != right).
/// - Finally iterate all gates and rebuild their open/closed state. This is brute force (we could
///   also keep an edge set of 'possibly changed' gates in the above step) but this might actually
///   be faster. It's also trivially-parallelizable although again, the overhead is likely not worth
///   it.
use crate::substrate::IntegratedCircuit;

use super::Path;

/// Encapsulates all the state needed to run a simulation on a compiled IC.
pub struct Simulation {
    gate_states: Vec<bool>,
    gates: Vec<Gate>,
    path_states: Vec<bool>,
    paths: Vec<Path>,
    pins: Vec<Pin>,
}

struct Gate {
    is_npn: bool,
    base_path_idx: usize,
    left_ec_path_idx: usize,
    right_ec_path_idx: usize,
}

struct Pin {
    path_idx: usize,
}

impl Simulation {
    pub fn new(ic: &IntegratedCircuit) -> Self {
        todo!()
    }
}
