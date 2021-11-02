use std::{
    collections::{HashMap, VecDeque},
    iter::FromIterator,
};

use glam::IVec2;

use crate::{log, substrate::MosfetPart, utils::ChunkedHashMap};

use super::{
    atom::Cell,
    pin::{PinModule, PinModuleState, PinState},
    Atom, Pin, Placement,
};

/// Stores atoms, indexed across several dimensions. Focused on fast reads, at the expense of slow
/// mutations and memory size.
#[derive(Default)]
pub struct IntegratedCircuit {
    // Atoms
    cell_lookup_by_loc: ChunkedHashMap<Cell>,

    // Traces
    traces: Vec<Vec<Atom>>,
    trace_lookup_by_atom: HashMap<Atom, usize>,

    // Gates
    gates: Vec<Gate>,

    // Pins
    pin_modules: Vec<PinModule>,
    pin_lookup_by_loc: ChunkedHashMap<Pin>,
}

#[derive(Debug, Clone, Copy)]
pub struct Gate {
    pub is_npn: bool,
    pub base_trace: usize,
    pub left_ec_trace: usize,
    pub right_ec_trace: usize,
}

#[derive(Debug, Clone)]
pub struct SimTickParams {
    pub tick: usize,
}

#[derive(Debug, Clone)]
pub struct SimIcState {
    pub params: SimTickParams,
    pub gate_states: Vec<bool>,
    pub trace_states: Vec<bool>,
    pub pin_module_states: Vec<PinModuleState>,
    pub pin_states: Vec<Vec<PinState>>,
}

impl IntegratedCircuit {
    pub fn new(cell_lookup_by_loc: ChunkedHashMap<Cell>, pin_modules: Vec<PinModule>) -> Self {
        let mut ic = Self {
            cell_lookup_by_loc,
            pin_modules,
            ..Default::default()
        };

        for pin_module in ic.pin_modules.iter() {
            for pin in pin_module.get_pins() {
                ic.pin_lookup_by_loc.set_cell(pin.cell_loc, pin);
            }
        }

        ic.rebuild_traces_and_gates();
        ic
    }

    pub fn commit_cell_changes(&mut self, changes: Vec<(IVec2, Cell)>) {
        for (loc, change) in changes {
            self.cell_lookup_by_loc.set_cell(loc, change);
        }

        self.rebuild_traces_and_gates();
    }

    #[inline]
    pub fn get_cell_by_location(&self, cell_loc: &IVec2) -> Option<&Cell> {
        self.cell_lookup_by_loc.get_cell(&cell_loc)
    }

    #[inline]
    pub fn get_cell_chunk_by_chunk_location(
        &self,
        chunk_loc: &IVec2,
    ) -> Option<&HashMap<IVec2, Cell>> {
        self.cell_lookup_by_loc.get_chunk(&chunk_loc)
    }

    pub fn chunk_locs<'a>(&'a self) -> impl Iterator<Item = &'a IVec2> {
        self.cell_lookup_by_loc.chunk_lookup_by_chunk_idx.keys()
    }

    pub fn pin_modules<'a>(&'a self) -> impl Iterator<Item = &'a PinModule> {
        self.pin_modules.iter()
    }

    #[inline]
    pub fn get_trace_handle_by_atom(&self, atom: &Atom) -> Option<usize> {
        self.trace_lookup_by_atom.get(atom).cloned()
    }

    #[inline]
    pub fn get_pin_chunk_by_chunk_location(
        &self,
        chunk_loc: &IVec2,
    ) -> Option<&HashMap<IVec2, Pin>> {
        self.pin_lookup_by_loc.get_chunk(chunk_loc)
    }

    pub fn add_pin_module(&mut self, pin_module: PinModule) {
        for pin in pin_module.get_pins() {
            self.pin_lookup_by_loc.set_cell(pin.cell_loc, pin);
        }

        self.pin_modules.push(pin_module);

        self.rebuild_traces_and_gates();
    }

    pub fn build_new_sim_state(&self) -> SimIcState {
        SimIcState {
            params: SimTickParams { tick: 0 },
            gate_states: vec![false; self.gates.len()],
            trace_states: vec![false; self.traces.len()],
            pin_module_states: self
                .pin_modules
                .iter()
                .map(|m| PinModuleState::instantiate(m))
                .collect(),
            pin_states: self
                .pin_modules
                .iter()
                .map(|m| vec![Default::default(); m.get_pins().len()])
                .collect(),
        }
    }

    pub fn step_simulation_state(&self, previous_state: SimIcState) -> SimIcState {
        let mut state = SimIcState {
            params: previous_state.params,
            gate_states: previous_state.gate_states,
            trace_states: vec![false; previous_state.trace_states.len()],
            pin_module_states: previous_state.pin_module_states,
            pin_states: previous_state.pin_states,
        };

        // Pull input pin modules and update corresponding traces.
        for (i, pin_module) in self.pin_modules.iter().enumerate() {
            // Update the module.
            let pin_states = &mut state.pin_states[i];
            state.pin_module_states[i].update_pin_state_inputs(pin_states, &state.params);

            // Pull the pin states and write them to their corresponding trace.
            for (j, pin) in pin_module.get_pins().iter().enumerate() {
                // Technically an I/O pin doesn't need to correspond with metal in the cell.
                if let Some(cell) = self.get_cell_by_location(&pin.cell_loc) {
                    for atom in cell {
                        // Find the metal atom in the cell (if any)
                        if atom.metal == Placement::NONE {
                            continue;
                        }

                        // Get the corresponding trace and update it.
                        let trace_idx = *self
                            .trace_lookup_by_atom
                            .get(atom)
                            .expect("Trace index by atom missing for metal atom");

                        state.trace_states[trace_idx] |= pin_states[j].input_high;
                    }
                }
            }
        }

        // Propagate 'high' signals through the graph.
        let mut change = true;
        while change {
            change = false;

            for i in 0..self.gates.len() {
                // If the gate isn't open, ignore it.
                if !state.gate_states[i] {
                    continue;
                }

                let gate = self.gates[i];
                let left = state.trace_states[gate.left_ec_trace];
                let right = state.trace_states[gate.right_ec_trace];
                let high = left || right;
                change |= left != right;
                state.trace_states[gate.left_ec_trace] = high;
                state.trace_states[gate.right_ec_trace] = high;
            }
        }

        // Update gate states
        for (i, gate) in self.gates.iter().enumerate() {
            let base = state.trace_states[gate.base_trace];

            state.gate_states[i] = if gate.is_npn { base } else { !base };
        }

        state.params.tick += 1;
        state
    }

    fn rebuild_traces_and_gates(&mut self) {
        self.traces.clear();
        self.trace_lookup_by_atom.clear();
        self.gates.clear();

        // The 0 trace is reserved as the null-trace.
        self.traces.push(vec![]);

        // Traces (and thus gates) have to be explored breadth-first off I/O pins. Reason being,
        // gates only hold back-references to traces when calculating their new states
        // (open/closed). Breadth first traversal means that gates are already in the order in which
        // they need to be updated and thus don't need to hold forward references to descendants in
        // the graph.

        // Seed the edge-set with all atoms attached to I/O pins.
        let mut edge_set: VecDeque<Atom> = self
            .pin_modules
            .iter()
            .flat_map(|m| m.get_pins())
            .map(|p| {
                self.get_cell_by_location(&p.cell_loc)
                    .map(|c| c.iter().find(|a| a.metal != Placement::NONE).cloned())
            })
            .filter(|a| matches!(a, Some(Some(_))))
            .map(|a| a.unwrap().unwrap())
            .collect();

        // Because it's easier to build up all traces first, we store all base atoms as we go along
        // and build up gates later from them. They will also implicitly be in breadth-first order.
        let mut base_atoms = vec![];

        while edge_set.len() > 0 {
            let atom = edge_set.pop_front().unwrap();

            // Check if the atom was already explored.
            if self.trace_lookup_by_atom.contains_key(&atom) {
                continue;
            }

            // It's a new trace, search the trace atoms and add them.
            let mut trace = vec![];
            let trace_idx = self.traces.len();
            let mut trace_edge_set = VecDeque::from_iter([atom.clone()]);

            while trace_edge_set.len() > 0 {
                let atom = trace_edge_set.pop_front().unwrap();

                if self.trace_lookup_by_atom.contains_key(&atom) {
                    continue;
                }

                self.trace_lookup_by_atom.insert(atom.clone(), trace_idx);

                // Add atom neighbors of metal and si. Note that this works because atoms are
                // self-descriptive, ie each atom always describes it's conductive neighbors
                // irrespective of it's membership to a MOSFET. Likewise, MOSFET parts do not
                // conductively connect together so they will not be explored.
                for dir in (atom.metal | atom.si).cardinal_vectors() {
                    trace_edge_set.extend(
                        self.get_cell_by_location(&(atom.cell_loc + dir))
                            .unwrap()
                            .iter()
                            .filter(|o| atom.neighbor_of(o)),
                    );
                }

                trace.push(atom);

                if atom.mosfet_part == MosfetPart::Base {
                    base_atoms.push(atom.clone());
                }

                // If the atom is itself part of a MOSFET, then we add the other MOSFET part atoms
                // to the global edge set before continuing. We don't need to worry about duplicates
                // because that is handled at the start of the outer loop.
                if atom.mosfet_part != MosfetPart::None {
                    edge_set.extend(
                        self.get_cell_by_location(&atom.cell_loc)
                            .unwrap()
                            .iter()
                            .filter(|a| a.mosfet_part != MosfetPart::None),
                    );
                }
            }

            self.traces.push(trace);
        }

        // Now we can build up gates as all traces have already been created. The `base_atoms` vec
        // is already in breadth-first other.
        for base_atom in base_atoms {
            let mut gate = Gate {
                is_npn: !base_atom.is_si_n,
                base_trace: *self.trace_lookup_by_atom.get(&base_atom).unwrap(),
                left_ec_trace: usize::MAX,
                right_ec_trace: usize::MAX,
            };

            // Find the EC atoms
            for atom in self.get_cell_by_location(&base_atom.cell_loc).unwrap() {
                match atom.mosfet_part {
                    MosfetPart::LeftEC => {
                        gate.left_ec_trace = *self.trace_lookup_by_atom.get(&atom).unwrap()
                    }
                    MosfetPart::RightEC => {
                        gate.right_ec_trace = *self.trace_lookup_by_atom.get(&atom).unwrap()
                    }
                    _ => {}
                }
            }

            debug_assert!(gate.left_ec_trace != usize::MAX && gate.right_ec_trace != usize::MAX);

            self.gates.push(gate);
        }

        log!(
            "Traces: {}, Gates: {}",
            self.traces.len() - 1,
            self.gates.len()
        );
    }
}
