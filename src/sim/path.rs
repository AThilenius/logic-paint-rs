use std::collections::HashSet;

use glam::IVec2;

use crate::{
    sim::atom::AtomType,
    substrate::{IntegratedCircuit, Metal, Silicon},
};

use super::atom::Atom;

#[derive(Debug)]
pub struct Path {
    pub atoms: Vec<Atom>,
}

impl Path {
    pub fn explore_atom_and_update_cell_paths(
        ic: &mut IntegratedCircuit,
        explored: &mut HashSet<(IVec2, AtomType)>,
        seed_atom: Atom,
        path_idx: usize,
    ) -> Path {
        let mut atoms = vec![];
        let mut edge_set: Vec<Atom> = vec![seed_atom];

        while edge_set.len() > 0 {
            let atom = edge_set.pop().unwrap();
            if explored.contains(&(atom.src_loc, atom.atom_type)) {
                continue;
            }
            explored.insert((atom.src_loc, atom.atom_type));
            atoms.push(atom);

            let loc = atom.src_loc;
            let cell = ic.get_cell(&loc).unwrap();

            match (atom.atom_type, cell.si, cell.metal) {
                (AtomType::TerminalMosfetBase { .. }, Silicon::Mosfet { gate_dirs, .. }, _) => {
                    // MOSFET Base pin always connects to non-metal right "above" the gate and
                    // nothing else. This is done to keep terminal connections single-sibling.
                    edge_set.push(Atom {
                        src_loc: loc,
                        path: path_idx,
                        atom_type: AtomType::NonMetal,
                    });

                    // However, the easiest place to explore connecting Non-Metal for the Base pin
                    // is here, so we add that as well. Vias cannot connect directly to a MOSFET, so
                    // the metal layer is irrelevant.
                    for offset in gate_dirs.get_offsets() {
                        edge_set.push(Atom {
                            src_loc: loc + offset,
                            path: path_idx,
                            atom_type: AtomType::NonMetal,
                        });
                    }

                    // We do not explore the Emitter/Collector atoms because they are part of
                    // another Path. They will be explored in the Network search.
                }
                (
                    AtomType::TerminalMosfetEC { dir, .. },
                    Silicon::Mosfet { .. },
                    _,
                ) => {
                    // An emitter or collector of a MOSFET *always* connects to non-metal in an
                    // adjacent cell in the `dir` direction. Remember, that's how they are drawn in
                    // the first place.
                    edge_set.push(Atom { src_loc: loc + dir, path: path_idx, atom_type: AtomType::NonMetal });
                }
                (
                    AtomType::NonMetal,
                    Silicon::NP { dirs, .. },
                    metal,
                ) => {
                    // Non-metal atom for single-layer Silicon can connect to adjacent like-type
                    // silicon, including MOSFETs.
                    for offset in dirs.get_offsets() {
                        // In the MOSFET case, we need to discriminate connecting to the NonMetal
                        // "above" the gate and connecting to an E/C.
                        match ic.get_cell(&(loc + offset)).unwrap().si {
                            Silicon::Mosfet { ec_dirs, is_npn, .. } => {
                                // The neighbor is indeed a MOSFET, are we connecting to an E/C pin?
                                if ec_dirs.get_offsets().iter().any(|o| offset + *o == IVec2::ZERO) {
                                    // Yep. There is already a non-metal at the cell adjacent the
                                    // MOSFET (this atom) so we just add a TerminalMosfetEC.
                                    edge_set.push(Atom {
                                        src_loc: loc + offset,
                                        path: path_idx,
                                        atom_type: AtomType::TerminalMosfetEC { is_npn, dir: -offset }
                                    });
                                } else {
                                    // Nope, we are connecting to the base. Just add the NonMetal
                                    // coincident the MOSFET, the case below this one will generate
                                    // the TerminalMosfetBase atom.
                                    edge_set.push(Atom {
                                        src_loc: loc + offset,
                                        path: path_idx,
                                        atom_type: AtomType::NonMetal
                                    });
                                }
                            }
                            Silicon::NP {..} => {
                                // Otherwise we aren't connecting to a MOSFET, it's just a normal
                                // single-layer silicon (otherwise it wouldn't be in `dirs`).
                                edge_set.push(Atom {
                                    src_loc: loc + offset,
                                    path: path_idx,
                                    atom_type: AtomType::NonMetal,
                                });
                            }
                            _ => {},
                        }
                    }

                    if let Metal::Trace { has_via: true, .. } = metal {
                        edge_set.push(Atom {
                            src_loc: loc,
                            path: path_idx,
                            atom_type: AtomType::Metal
                        });
                    }
                }
                (
                    AtomType::NonMetal,
                    Silicon::Mosfet { is_npn, gate_dirs: base_dirs, .. },
                    _
                ) => {
                    // NonMetal coincident with a MOSFET is connected to the TerminalMosfetBase, as
                    // well as any silicon connected to the base.
                    edge_set.push(Atom {
                        src_loc: loc,
                        path: path_idx,
                        atom_type: AtomType::TerminalMosfetBase { is_npn }
                    });

                    for offset in base_dirs.get_offsets() {
                        edge_set.push(Atom { src_loc: loc + offset, path: path_idx, atom_type: AtomType::NonMetal });
                    }
                }
                (AtomType::Metal, Silicon::NP {.. }, Metal::Trace { has_via: true, dirs, .. }) => {
                    // Via connection
                    edge_set.push(Atom {
                        src_loc: loc,
                        path: path_idx,
                        atom_type: AtomType::NonMetal
                    });

                    // Other metal connections
                    for offset in dirs.get_offsets() {
                        edge_set.push(Atom {
                            src_loc: loc + offset,
                            path: path_idx,
                            atom_type: AtomType::Metal
                        });
                    }
                },
                (AtomType::Metal, _, Metal::Trace { dirs, ..}) => {
                    for offset in dirs.get_offsets() {
                        edge_set.push(Atom {
                            src_loc: loc + offset,
                            path: path_idx,
                            atom_type: AtomType::Metal
                        });
                    }
                },
                _ => panic!(
                    "Unsupported search tipple at {} could indicate an invalid ic: (\n  {:?}, \n  {:?}, \n  {:?}\n)",
                    loc, atom.atom_type, cell.si, cell.metal
                ),
            }
        }

        // For code simplicity, update cell path's after the fact or it muddies the atom exploration
        // logic.
        for atom in atoms.iter() {
            let loc = atom.src_loc;
            let mut cell = ic.get_cell(&loc).unwrap();

            match (atom.atom_type, &mut cell.metal, &mut cell.si) {
                (AtomType::Metal, Metal::Trace { path, .. }, _)
                | (AtomType::NonMetal, _, Silicon::NP { path, .. })
                | (AtomType::TerminalMosfetBase { .. }, _, Silicon::Mosfet { path, .. }) => {
                    // TODO: Consider splitting the Emitter/Collector 'active' for rendering.
                    *path = path_idx;
                }
                _ => {
                    // All other atom types are redundant.
                }
            }

            ic.set_cell(loc, cell);
        }

        Path { atoms }
    }
}
