use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::FromIterator,
};

use glam::IVec2;

use super::{atom::Cell, Atom};

/// Stores atoms, indexed across several dimensions. Focused on fast reads, at the expense of slow
/// mutations and memory size.
#[derive(Debug, Default)]
pub struct IntegratedCircuit {
    atoms: HashSet<Atom>,
    cell_lookup_by_loc: HashMap<IVec2, Cell>,

    traces: Vec<Vec<Atom>>,
    trace_lookup_by_atom: HashMap<Atom, usize>,
}

impl IntegratedCircuit {
    pub fn commit_cell_changes(&mut self, changes: Vec<(IVec2, Cell)>) {
        for (loc, change) in changes {
            if change.len() == 0 {
                // Delete the cell entirely.
                if let Some(previous_cell) = self.cell_lookup_by_loc.remove(&loc) {
                    for atom in previous_cell {
                        self.atoms.remove(&atom);
                    }
                }
            } else {
                if let Some(previous_cell) = self.cell_lookup_by_loc.insert(loc, change.clone()) {
                    for atom in previous_cell {
                        self.atoms.remove(&atom);
                    }
                }
                self.atoms.extend(change);
            }
        }

        self.rebuild_traces();
    }

    pub fn get_cell_by_location(&self, cell_loc: IVec2) -> Option<Cell> {
        self.cell_lookup_by_loc.get(&cell_loc).cloned()
    }

    fn rebuild_traces(&mut self) {
        self.traces.clear();
        self.trace_lookup_by_atom.clear();

        for atom in self.atoms.iter() {
            // Atom was already explored.
            if self.trace_lookup_by_atom.contains_key(atom) {
                continue;
            }

            // It's a new trace, search the trace atoms and add them.
            let mut trace = vec![];
            let trace_idx = self.traces.len();
            let mut edge_set = VecDeque::from_iter([*atom]);

            while edge_set.len() > 0 {
                let atom = edge_set.pop_front().unwrap();

                if self.trace_lookup_by_atom.contains_key(&atom) {
                    continue;
                }

                self.trace_lookup_by_atom.insert(atom.clone(), trace_idx);

                // Add atom neighbors of metal and si. Note that this works because atoms are
                // self-descriptive, ie each atom always describes it's conductive neighbors
                // irrespective of it's membership to a MOSFET. Likewise, MOSFET parts do not
                // conductively connect together so they will not be explored.
                for dir in (atom.metal | atom.si).cardinal_vectors() {
                    edge_set.extend(
                        self.get_cell_by_location(atom.cell_loc + dir)
                            .unwrap()
                            .iter()
                            .filter(|o| atom.neighbor_of(o)),
                    );
                }

                trace.push(atom);
            }

            self.traces.push(trace);
        }
    }
}
