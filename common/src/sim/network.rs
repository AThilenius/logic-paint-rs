use std::collections::HashSet;

use crate::{
    canvas::{Canvas, Metal, Silicon},
    sim::atom::{Atom, AtomType},
};

use super::path::Path;

/// While a Canvas is used for editing and rendering, a Network is used to generate simulation state
/// and inspect logic layout. These are generated from a Canvas. A `Network` is essentially a list
/// of disconnected conductor paths within a single layout. The conductor paths are "terminated" at
/// a specific part of a cell, either a MOSFET emitter/collector, a MOSFET gate, or an IO pin.
#[derive(Default, Debug)]
pub struct Network {
    /// Set of conductive paths in a compiled canvas.
    pub paths: Vec<Path>,

    /// Drive counts for each path (matching index as `paths`).
    pub path_dc: Vec<u16>,
}

impl Network {
    pub fn compile_canvas(canvas: &Canvas) -> Network {
        let mut network = Network::default();
        let mut explored: HashSet<Atom> = HashSet::new();

        // Seed the edge set with IO pin terminal atoms.
        // TODO: Consider an idex for this.
        let mut edge_set: Vec<Atom> = canvas
            .cells
            .iter()
            .filter(|(_, c)| matches!(c.metal, Metal::IO { .. }))
            .map(|(l, _)| Atom {
                src_loc: *l,
                atom_type: AtomType::TerminalIoPin,
            })
            .collect();

        // Breadth-first search of all paths that connect to at least one IO pin.
        while edge_set.len() > 0 {
            let atom = edge_set.pop().unwrap();
            if explored.contains(&atom) {
                continue;
            }

            let path = Path::explore_atom(&mut explored, atom, &canvas);

            // Collect all terminal atoms from the path and add connecting MOSFET atoms to the
            // explore set.
            for atom in path.atoms.iter() {
                let cell = canvas.cells.get(&atom.src_loc).unwrap();
                match (atom.atom_type, cell.si) {
                    (AtomType::TerminalMosfetBase { is_npn }, Silicon::Mosfet { ec_dirs, .. }) => {
                        // Add both Emitter/Collector atoms.
                        for offset in ec_dirs.get_offsets() {
                            edge_set.push(Atom {
                                src_loc: atom.src_loc,
                                atom_type: AtomType::TerminalMosfetEC {
                                    is_npn,
                                    dir: offset,
                                },
                            });
                        }
                    }
                    (
                        AtomType::TerminalMosfetEC { is_npn, dir, .. },
                        Silicon::Mosfet { ec_dirs, .. },
                    ) => {
                        for offset in ec_dirs.get_offsets() {
                            if offset != dir {
                                edge_set.push(Atom {
                                    src_loc: atom.src_loc,
                                    atom_type: AtomType::TerminalMosfetEC {
                                        is_npn,
                                        dir: offset,
                                    },
                                });
                            }
                        }
                    }
                    _ => {
                        // We don't care about all other atoms. MOSFETs are the only atoms that
                        // bridge Paths.
                    }
                }
            }

            network.paths.push(path);
        }

        network
    }
}
