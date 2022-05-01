use std::{
    collections::{HashMap, VecDeque},
    iter::FromIterator,
};

use crate::{
    buffer::Buffer,
    coords::{CellCoord, ChunkCoord, CHUNK_SIZE},
    log,
    upc::{Bit, Metal, NormalizedCell, Silicon},
};

pub struct CompilerResults {
    pub traces: Vec<Vec<Atom>>,
    pub trace_lookup_by_atom: HashMap<Atom, usize>,
    pub gates: Vec<Gate>,
    pub trace_to_cell_part_index_by_chunk: HashMap<ChunkCoord, Vec<CellPartToTrace>>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Atom {
    pub coord: CellCoord,
    pub part: CellPart,
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
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

#[derive(Copy, Clone)]
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

impl CompilerResults {
    pub fn from_buffer(buffer: &Buffer) -> CompilerResults {
        let mut res = CompilerResults {
            // The 0 trace is reserved to mean the 'null' trace.
            traces: vec![vec![]],
            trace_lookup_by_atom: HashMap::new(),
            gates: vec![],
            trace_to_cell_part_index_by_chunk: HashMap::new(),
        };

        // Traces (and thus gates) have to be explored breadth-first off I/O pins. Reason being,
        // gates only hold back-references to traces when calculating their new states
        // (open/closed). Breadth first traversal means that gates are already in the order in which
        // they need to be updated and thus don't need to hold forward references to descendants in
        // the graph.
        //
        // Ie. The edge set is a queue (breadth-first) and is seeded with I/O pins.
        let mut edge_set: VecDeque<Atom> = buffer
            .get_modules()
            .iter()
            .flat_map(|m| m.get_pins())
            .map(|c| Atom {
                coord: c.coord.clone(),
                part: CellPart::Metal,
            })
            .collect();

        // Compiled gates get connected to their respective EC traces by index, but we still need to
        // explore gates breadth-first, so it's easier to build up all traces (while storing the
        // location of all gates/base atoms we find) then build up the compiled gates.
        let mut base_atoms = vec![];

        while let Some(atom) = edge_set.pop_front() {
            if res.trace_lookup_by_atom.contains_key(&atom) {
                continue;
            }

            let mut trace = vec![];
            let trace_idx = res.traces.len();
            let mut trace_edge_set = VecDeque::from_iter([atom.clone()]);

            while let Some(atom) = trace_edge_set.pop_front() {
                if res.trace_lookup_by_atom.contains_key(&atom) {
                    continue;
                }

                res.trace_lookup_by_atom.insert(atom.clone(), trace_idx);
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
                        },
                    ) => {
                        // Follow conductive Si trace
                        trace_edge_set.extend(placement.cardinal_vectors().iter().map(|v| Atom {
                            coord: CellCoord(atom.coord.0 + *v),
                            part: CellPart::Si,
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
                        },
                    ) => {
                        // This atom is implicitly the gate atom. Store it in the `base_atoms` set
                        // so that we can explore off it later.
                        base_atoms.push(atom.clone());

                        // Follow conductive Si trace off the Gate
                        trace_edge_set.extend(gate_placement.cardinal_vectors().iter().map(|v| {
                            Atom {
                                coord: CellCoord(atom.coord.0 + *v),
                                part: CellPart::Si,
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
                        // this trace).
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
                        trace_edge_set.push_back(Atom {
                            coord: if ec_placement.left {
                                (atom.coord.0.x - 1, atom.coord.0.y).into()
                            } else {
                                (atom.coord.0.x, atom.coord.0.y + 1).into()
                            },
                            part: CellPart::Si,
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
                        trace_edge_set.push_back(Atom {
                            coord: if ec_placement.right {
                                (atom.coord.0.x + 1, atom.coord.0.y).into()
                            } else {
                                (atom.coord.0.x, atom.coord.0.y - 1).into()
                            },
                            part: CellPart::Si,
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

            res.traces.push(trace);
        }

        // Now that we have all traces built up, we can create the Gates with back-references to
        // trace index on their EC atoms.
        for atom in base_atoms {
            let cell = buffer.get_cell(atom.coord);
            let gate = Gate {
                is_npn: cell.get_bit(Bit::SI_N),
                base_trace: *res.trace_lookup_by_atom.get(&atom).unwrap(),
                left_ec_trace: *res
                    .trace_lookup_by_atom
                    .get(&Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::EcUpLeft,
                    })
                    .unwrap(),
                right_ec_trace: *res
                    .trace_lookup_by_atom
                    .get(&Atom {
                        coord: atom.coord.clone(),
                        part: CellPart::EcDownRight,
                    })
                    .unwrap(),
            };

            res.gates.push(gate);
        }

        // Create an index for quickly copying trace states over to a BufferMask.
        for (chunk_coord, chunk) in buffer.get_base_chunks() {
            let mut trace_indexes = vec![];

            // TODO: This can be made a lot more efficient if compilation times ever become a
            // problem.
            let mut i = 0;
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let coord = CellCoord::from_offset_into_chunk(&chunk_coord, x, y);

                    // Skip empty cells.
                    if chunk.get_cell(coord.clone()) == Default::default() {
                        continue;
                    }

                    trace_indexes.push(CellPartToTrace {
                        cell_index_in_chunk: i,
                        metal_trace: res
                            .trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::Metal,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        si_trace: res
                            .trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::Si,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        left_ec_trace: res
                            .trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::EcUpLeft,
                            })
                            .cloned()
                            .unwrap_or_default(),
                        right_ec_trace: res
                            .trace_lookup_by_atom
                            .get(&Atom {
                                coord: coord.clone(),
                                part: CellPart::EcDownRight,
                            })
                            .cloned()
                            .unwrap_or_default(),
                    });

                    i += 1;
                }
            }

            res.trace_to_cell_part_index_by_chunk
                .insert(*chunk_coord, trace_indexes);
        }

        log!(
            "Traces: {}, Gates: {}",
            res.traces.len() - 1,
            res.gates.len()
        );

        res
    }
}
