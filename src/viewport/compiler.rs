use std::{
    collections::{HashMap, VecDeque},
    iter::FromIterator,
};

use wasm_bindgen::prelude::*;

use crate::{
    coords::{CellCoord, ChunkCoord, CHUNK_SIZE, LOG_CHUNK_SIZE},
    log,
    upc::{Bit, Metal, NormalizedCell, Silicon},
    viewport::buffer::Buffer,
};

#[wasm_bindgen]
pub struct CompilerResults {
    #[wasm_bindgen(skip)]
    pub traces: Vec<Vec<Atom>>,
    #[wasm_bindgen(skip)]
    pub trace_lookup_by_atom: HashMap<Atom, usize>,
    #[wasm_bindgen(skip)]
    pub gates: Vec<Gate>,
    #[wasm_bindgen(skip)]
    pub trace_to_cell_part_index_by_chunk: HashMap<ChunkCoord, Vec<CellPartToTrace>>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[wasm_bindgen]
pub struct Atom {
    pub coord: CellCoord,
    pub part: CellPart,
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[wasm_bindgen]
pub enum CellPart {
    // Metal (including the Via)
    Metal = 0,

    // Best described as "anything non-metal a Via can attach to". So Si traces, or gates. This is
    // the inverse of what we consider Si for things like the NormalizedCell.
    Si = 1,

    // Up or left EC (never both because MOSFETS can't be drawn that way).
    EcUpLeft = 2,

    // Down or right EC (never both because MOSFETS can't be drawn that way).
    EcDownRight = 3,
}

#[derive(Copy, Clone, Debug)]
pub struct CellPartToTrace {
    pub cell_index_in_chunk: usize,
    pub metal_trace: usize,
    pub si_trace: usize,
    pub left_ec_trace: usize,
    pub right_ec_trace: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Gate {
    pub is_npn: bool,
    pub base_trace: usize,
    pub left_ec_trace: usize,
    pub right_ec_trace: usize,
}

#[wasm_bindgen]
impl CompilerResults {
    #[wasm_bindgen(constructor)]
    pub fn from_buffer(buffer: &Buffer) -> CompilerResults {
        // Traces (and thus gates) have to be explored breadth-first off I/O pins. Reason being,
        // gates only hold back-references to traces when calculating their new states
        // (open/closed). Breadth first traversal means that gates are already in the order in which
        // they need to be updated and thus don't need to hold forward references to descendants in
        // the graph.
        //
        // Ie. The edge set is a queue (breadth-first) and is seeded with I/O pins.
        let mut edge_set: VecDeque<Atom> = buffer
            .modules
            .values()
            .flat_map(|m| m.get_pin_coords())
            .map(|coord| Atom {
                coord,
                part: CellPart::Metal,
            })
            .collect();

        // Compiled gates get connected to their respective EC traces by index, but we still need to
        // explore gates breadth-first, so it's easier to build up all traces (while storing the
        // location of all gates/base atoms we find) then build up the compiled gates.
        let mut base_atoms = vec![];

        // Note: the zero trace is reserved to mean the "null" trace.
        let mut traces = vec![vec![]];
        let mut trace_lookup_by_atom = HashMap::new();

        while let Some(atom) = edge_set.pop_front() {
            if trace_lookup_by_atom.contains_key(&atom) {
                continue;
            }

            CompilerResults::explore_trace(
                buffer,
                atom,
                &mut trace_lookup_by_atom,
                &mut traces,
                &mut edge_set,
                &mut base_atoms,
            );
        }

        let mut gates = vec![];
        let mut trace_to_cell_part_index_by_chunk = HashMap::new();

        // Now that we have all traces built up, we can create the Gates with back-references to
        // trace index on their EC atoms.
        for atom in base_atoms {
            let cell = buffer.get_cell(atom.coord);
            let gate = Gate {
                is_npn: cell.get_bit(Bit::SI_N),
                base_trace: *trace_lookup_by_atom.get(&atom).unwrap(),
                left_ec_trace: *trace_lookup_by_atom
                    .get(&Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::EcUpLeft,
                    })
                    .unwrap(),
                right_ec_trace: *trace_lookup_by_atom
                    .get(&Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::EcDownRight,
                    })
                    .unwrap(),
            };

            gates.push(gate);
        }

        // Create an index for quickly copying trace states over to a BufferMask.
        for (chunk_coord, chunk) in &buffer.chunks {
            let mut trace_indexes = vec![];

            // TODO: This can be made a lot more efficient if compilation times ever become a
            // problem.
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let coord = CellCoord::from_offset_into_chunk(&chunk_coord, x, y);

                    // Skip empty cells.
                    if chunk.get_cell(coord.clone()) == Default::default() {
                        continue;
                    }

                    let i = (y << LOG_CHUNK_SIZE) + x;

                    trace_indexes.push(CellPartToTrace {
                        cell_index_in_chunk: i,
                        metal_trace: trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::Metal,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        si_trace: trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::Si,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        left_ec_trace: trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::EcUpLeft,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        right_ec_trace: trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::EcDownRight,
                            })
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
            }

            trace_to_cell_part_index_by_chunk.insert(*chunk_coord, trace_indexes);
        }

        CompilerResults {
            traces,
            trace_lookup_by_atom,
            gates,
            trace_to_cell_part_index_by_chunk,
        }
    }

    pub fn get_trace_atoms(buffer: &Buffer, edge_atom: Atom) -> Vec<Atom> {
        let mut edge_set = VecDeque::new();
        edge_set.push_back(edge_atom);

        let mut base_atoms = vec![];
        let mut traces = vec![vec![]];
        let mut trace_lookup_by_atom = HashMap::new();

        CompilerResults::explore_trace(
            buffer,
            edge_atom,
            &mut trace_lookup_by_atom,
            &mut traces,
            &mut edge_set,
            &mut base_atoms,
        );

        traces[1].clone()
    }

    fn explore_trace(
        buffer: &Buffer,
        atom: Atom,
        trace_lookup_by_atom: &mut HashMap<Atom, usize>,
        traces: &mut Vec<Vec<Atom>>,
        edge_set: &mut VecDeque<Atom>,
        base_atoms: &mut Vec<Atom>,
    ) {
        let mut trace = vec![];
        let trace_idx = traces.len();
        let mut trace_edge_set = VecDeque::from_iter([atom.clone()]);

        while let Some(atom) = trace_edge_set.pop_front() {
            if trace_lookup_by_atom.contains_key(&atom) {
                continue;
            }

            trace_lookup_by_atom.insert(atom.clone(), trace_idx);
            trace.push(atom.clone());

            let cell = buffer.get_cell(atom.coord);
            let norm: NormalizedCell = cell.into();

            match (atom.part, norm) {
                (
                    CellPart::Metal,
                    NormalizedCell {
                        metal: Metal::Trace { has_via, placement },
                        ..
                    },
                ) => {
                    trace_edge_set.extend(placement.cardinal_vectors().iter().map(|v| Atom {
                        coord: CellCoord(atom.coord.0 + *v),
                        part: CellPart::Metal,
                    }));

                    if has_via {
                        trace_edge_set.push_back(Atom {
                            coord: atom.coord.clone(),
                            part: CellPart::Si,
                        })
                    }
                }
                (
                    CellPart::Si,
                    NormalizedCell {
                        si: Silicon::NP { placement, .. },
                        metal,
                        ..
                    },
                ) => {
                    // Follow conductive Si trace. Could go into another trace, or into a MOSFET
                    trace_edge_set.extend(placement.cardinal_vectors().iter().map(|v| {
                        let coord = CellCoord(atom.coord.0 + *v);
                        let neighbor: NormalizedCell = buffer.get_cell(coord.clone()).into();

                        match neighbor.si {
                            // ECs are always up+down OR left+right, so we can just check the
                            // cardinal match of the EC to tell if we are running into the
                            // neighbor's gate or an EC.
                            Silicon::Mosfet { ec_placement, .. }
                                if ec_placement.has_cardinal(-*v) =>
                            {
                                Atom {
                                    coord,
                                    part: if (*v).x < 0 || (*v).y > 0 {
                                        CellPart::EcDownRight
                                    } else {
                                        CellPart::EcUpLeft
                                    },
                                }
                            }
                            _ => Atom {
                                coord,
                                part: CellPart::Si,
                            },
                        }
                    }));

                    // Follow Via
                    if let Metal::Trace { has_via: true, .. } = metal {
                        trace_edge_set.push_back(Atom {
                            coord: atom.coord.clone(),
                            part: CellPart::Metal,
                        })
                    }
                }
                (
                    CellPart::Si,
                    NormalizedCell {
                        si: Silicon::Mosfet { gate_placement, .. },
                        metal,
                        ..
                    },
                ) => {
                    // This atom is implicitly the gate atom. Store it in the `base_atoms` set
                    // so that we can explore off it later.
                    base_atoms.push(atom.clone());

                    // Follow conductive Si trace off the Gate
                    trace_edge_set.extend(gate_placement.cardinal_vectors().iter().map(|v| {
                        let coord = CellCoord(atom.coord.0 + *v);
                        let neighbor: NormalizedCell = buffer.get_cell(coord.clone()).into();

                        match neighbor.si {
                            // ECs are always up+down OR left+right, so we can just check the
                            // cardinal match of the EC to tell if we are running into the
                            // neighbor's gate or an EC.
                            Silicon::Mosfet { ec_placement, .. }
                                if ec_placement.has_cardinal(-*v) =>
                            {
                                Atom {
                                    coord,
                                    part: if (*v).x < 0 || (*v).y > 0 {
                                        CellPart::EcDownRight
                                    } else {
                                        CellPart::EcUpLeft
                                    },
                                }
                            }
                            _ => Atom {
                                coord,
                                part: CellPart::Si,
                            },
                        }
                    }));

                    // Follow Via
                    if let Metal::Trace { has_via: true, .. } = metal {
                        trace_edge_set.push_back(Atom {
                            coord: atom.coord.clone(),
                            part: CellPart::Metal,
                        })
                    }

                    // Add EC atoms to the OUTER edge set (they aren't conductively connected to
                    // this trace). We aren't walking to the neighbor's cell yet, so we don't
                    // care if the ECs are connected to Si, a Gate, or another EC. That's done
                    // below.
                    edge_set.extend([
                        Atom {
                            coord: atom.coord.clone(),
                            part: CellPart::EcUpLeft,
                        },
                        Atom {
                            coord: atom.coord.clone(),
                            part: CellPart::EcDownRight,
                        },
                    ])
                }
                (
                    CellPart::EcUpLeft,
                    NormalizedCell {
                        si: Silicon::Mosfet { ec_placement, .. },
                        ..
                    },
                ) => {
                    // Add the conductively connected Si
                    let neighbor_coord: CellCoord = if ec_placement.left {
                        (atom.coord.0.x - 1, atom.coord.0.y).into()
                    } else {
                        (atom.coord.0.x, atom.coord.0.y + 1).into()
                    };
                    let neighbor: NormalizedCell = buffer.get_cell(neighbor_coord.clone()).into();

                    trace_edge_set.push_back(match neighbor.si {
                        Silicon::Mosfet { ec_placement, .. }
                            if ec_placement.has_cardinal(neighbor_coord.0 - atom.coord.0) =>
                        {
                            Atom {
                                coord: neighbor_coord,
                                part: CellPart::EcDownRight,
                            }
                        }
                        _ => Atom {
                            coord: neighbor_coord,
                            part: CellPart::Si,
                        },
                    });

                    // Add the Gate Si to the OUTER edge set (they aren't conductively connected
                    // to this trace).
                    edge_set.push_back(Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::Si,
                    });
                }
                (
                    CellPart::EcDownRight,
                    NormalizedCell {
                        si: Silicon::Mosfet { ec_placement, .. },
                        ..
                    },
                ) => {
                    // Add the conductively connected Si
                    let neighbor_coord: CellCoord = if ec_placement.right {
                        (atom.coord.0.x + 1, atom.coord.0.y).into()
                    } else {
                        (atom.coord.0.x, atom.coord.0.y - 1).into()
                    };
                    let neighbor: NormalizedCell = buffer.get_cell(neighbor_coord.clone()).into();

                    trace_edge_set.push_back(match neighbor.si {
                        Silicon::Mosfet { ec_placement, .. }
                            if ec_placement.has_cardinal(neighbor_coord.0 - atom.coord.0) =>
                        {
                            Atom {
                                coord: neighbor_coord,
                                part: CellPart::EcUpLeft,
                            }
                        }
                        _ => Atom {
                            coord: neighbor_coord,
                            part: CellPart::Si,
                        },
                    });

                    // Add the Gate Si to the OUTER edge set (they aren't conductively connected
                    // to this trace).
                    edge_set.push_back(Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::Si,
                    });
                }
                _ => {
                    log!("Invalid atom+cell tuple: ({:#?}, {:#?})", atom, &norm);
                }
            }
        }

        traces.push(trace);
    }
}
