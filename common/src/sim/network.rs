use std::collections::HashSet;

use crate::{
    canvas::{Canvas, Silicon},
    sim::atom::{Atom, AtomType},
    utils::HilbertIndexing,
};

use super::path::Path;

/// While a Canvas is used for editing and rendering, a Network is used to generate simulation state
/// and inspect logic layout. These are generated from a Canvas. A `Network` is essentially a list
/// of disconnected conductor paths within a single layout. The conductor paths are "terminated" at
/// a specific part of a cell, either a MOSFET emitter/collector, a MOSFET gate, or an IO pin.
#[derive(Default, Debug)]
pub struct Network {
    pub paths: Vec<Path>,

    // Look-aside buffers for quick access to nodes within `paths`, as a tuples of (path index, atom
    // index).
    pub terminal_io_look_aside: Vec<(usize, usize)>,
    pub terminal_mosfet_base_look_aside: Vec<(usize, usize)>,
    pub terminal_mosfet_ec_look_aside: Vec<(usize, usize)>,
}

impl Network {
    pub fn compile_canvas(canvas: &Canvas) -> Network {
        let mut network = Network::default();
        let mut explored: HashSet<Atom> = HashSet::new();
        let mut edge_set: Vec<Atom> = vec![];

        // Seed the edge set with IO pin terminal atoms.
        for (loc, _) in canvas.io_pins.iter() {
            edge_set.push(Atom {
                src_loc: *loc,
                atom_type: AtomType::TerminalIoPin,
            });
        }

        // Breadth-first search of all paths that connect to at least one IO pin.
        while edge_set.len() > 0 {
            let atom = edge_set.pop().unwrap();
            if explored.contains(&atom) {
                continue;
            }

            let path = Path::explore_atom(&mut explored, atom, &canvas);
            let path_idx = network.paths.len();

            // Collect all terminal atoms from the path and add connecting MOSFET atoms to the
            // explore set.
            for (atom_idx, atom) in path.atoms.iter().enumerate() {
                let cell = canvas.cells.get(atom.src_loc);
                match (atom.atom_type, cell.si) {
                    (AtomType::TerminalMosfetBase { is_npn }, Silicon::Mosfet { ec_dirs, .. }) => {
                        network
                            .terminal_mosfet_base_look_aside
                            .push((path_idx, atom_idx));

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
                        network
                            .terminal_mosfet_ec_look_aside
                            .push((path_idx, atom_idx));

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
                    (AtomType::TerminalIoPin, _) => {
                        network.terminal_io_look_aside.push((path_idx, atom_idx));
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
