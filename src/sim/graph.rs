use std::collections::{hash_map::Entry, HashMap, VecDeque};

use glam::IVec2;
use smallvec::SmallVec;

use crate::{log, sim::AtomType, warn};

use super::Path;

#[derive(Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    edge_set: VecDeque<usize>,
    node_eval_count: u64,
    path_dc: Vec<u16>,
    loc_to_node_idx: HashMap<IVec2, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    node_type: NodeType,
    edges: SmallVec<[Edge; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    IO {
        extern_drv: bool,
        path_idx: usize,
    },
    NPN {
        base_path_idx: usize,
        collector_path_idx: usize,
    },
    PNP {
        base_path_idx: usize,
        collector_path_idx: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    to_node_idx: u32,
    edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    IoOut,
    Base,
    Collector,
}

impl Graph {
    pub fn new(paths: &Vec<Path>) -> Self {
        // Filter all terminal atoms
        let mut terminal_atoms = vec![];

        for (path_idx, path) in paths.iter().enumerate() {
            for atom in &path.atoms {
                if atom.atom_type != AtomType::NonMetal && atom.atom_type != AtomType::Metal {
                    terminal_atoms.push((path_idx, atom));
                }
            }
        }

        // Map each terminal atom's location (not the atom itself) to Node in the graph.
        let mut locs_to_nodes: HashMap<IVec2, Node> = HashMap::new();
        for (path_idx, terminal_atom) in terminal_atoms {
            let loc = terminal_atom.src_loc;

            match locs_to_nodes.entry(loc) {
                // If we already have a Node at the given loc it means it's an NPN/PNP and this is
                // the second atom pointing to that Node. We just need to record the other path idx.
                Entry::Occupied(existing) => {
                    match (terminal_atom.atom_type, &mut existing.into_mut().node_type) {
                        (
                            AtomType::TerminalMosfetEC { .. },
                            NodeType::NPN {
                                collector_path_idx, ..
                            }
                            | NodeType::PNP {
                                collector_path_idx, ..
                            },
                        ) => {
                            *collector_path_idx = path_idx;
                        }
                        (
                            AtomType::TerminalMosfetBase { .. },
                            NodeType::NPN { base_path_idx, .. }
                            | NodeType::PNP { base_path_idx, .. },
                        ) => {
                            *base_path_idx = path_idx;
                        }
                        _ => {}
                    }
                }
                // If the slot is vacant than this is the first time we've visited this location,
                // so make a Node fo rit.
                Entry::Vacant(slot) => match terminal_atom.atom_type {
                    AtomType::TerminalIoPin { .. } => {
                        slot.insert(Node {
                            edges: Default::default(),
                            node_type: NodeType::IO {
                                extern_drv: false,
                                path_idx,
                            },
                        });
                    }
                    AtomType::TerminalMosfetEC { is_npn: true, .. } => {
                        slot.insert(Node {
                            edges: Default::default(),
                            node_type: NodeType::NPN {
                                base_path_idx: usize::MAX,
                                collector_path_idx: path_idx,
                            },
                        });
                    }
                    AtomType::TerminalMosfetBase { is_npn: true, .. } => {
                        slot.insert(Node {
                            edges: Default::default(),
                            node_type: NodeType::NPN {
                                base_path_idx: path_idx,
                                collector_path_idx: usize::MAX,
                            },
                        });
                    }
                    AtomType::TerminalMosfetEC { is_npn: false, .. } => {
                        slot.insert(Node {
                            edges: Default::default(),
                            node_type: NodeType::PNP {
                                base_path_idx: usize::MAX,
                                collector_path_idx: path_idx,
                            },
                        });
                    }
                    AtomType::TerminalMosfetBase { is_npn: false, .. } => {
                        slot.insert(Node {
                            edges: Default::default(),
                            node_type: NodeType::PNP {
                                base_path_idx: path_idx,
                                collector_path_idx: usize::MAX,
                            },
                        });
                    }
                    _ => {}
                },
            }
        }

        // Nodes need to be turned into a Vec now as they are referenced by index. We also want a
        // lookup for location -> node_idx, so build that too.
        let mut loc_to_node_idx = HashMap::new();
        let mut nodes = vec![];
        for (loc, node) in locs_to_nodes.iter() {
            loc_to_node_idx.insert(*loc, nodes.len());
            nodes.push(node.clone());
        }

        // Now we can enumerate paths and build up edges between nodes.
        for path in paths {
            let atoms = &path.atoms;

            // Filter the edges into a few categories.
            let gate_nodes = atoms
                .iter()
                .filter(|atom| matches!(atom.atom_type, AtomType::TerminalMosfetBase { .. }));

            let ec_nodes = atoms
                .iter()
                .filter(|atom| matches!(atom.atom_type, AtomType::TerminalMosfetEC { .. }));

            let io_nodes = atoms
                .iter()
                .filter(|atom| matches!(atom.atom_type, AtomType::TerminalIoPin { .. }));

            for from_atom in ec_nodes.clone().chain(io_nodes.clone()) {
                let from_node_idx = *loc_to_node_idx.get(&from_atom.src_loc).unwrap();

                // Add a Base edge from all [E/C, IO] nodes to all Base nodes (one way)
                for to_atom in gate_nodes.clone() {
                    let to_node_idx = *loc_to_node_idx.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::Base,
                    });
                }

                // Add a Collector edge from all [E/C, IO] nodes to all other E/C nodes (two way)
                for to_atom in ec_nodes.clone() {
                    if from_atom.src_loc == to_atom.src_loc
                        && from_atom.atom_type == to_atom.atom_type
                    {
                        continue;
                    }

                    let to_node_idx = *loc_to_node_idx.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::Collector,
                    });
                }
            }

            for from_atom in ec_nodes.clone() {
                let from_node_idx = *loc_to_node_idx.get(&from_atom.src_loc).unwrap();

                // Add an IoOut edge from all E/C nodes to all IO nodes (one way)
                for to_atom in io_nodes.clone() {
                    let to_node_idx = *loc_to_node_idx.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::IoOut,
                    });
                }
            }

            // TODO: What about IO driving IO (clock sampling, for example).
        }

        log!("Compiled graph into the nodes: {:#?}", &nodes);

        Graph {
            nodes,
            edge_set: VecDeque::new(),
            node_eval_count: 0,
            path_dc: vec![0; paths.len()],
            loc_to_node_idx,
        }
    }

    pub fn set_io_drive_state(&mut self, loc: IVec2, driven: bool) {
        // Driving an I/O externally just means incrementing or decrementing the DC for all nodes
        // the I/O is connected to. If the DC switches from 0 <--> !0 then we also add the node to
        // the edge set. Start by modifying the node_idx node.
        let (driving, edges) = {
            let node_idx = *self.loc_to_node_idx.get(&loc).unwrap();
            let node = &mut self.nodes[node_idx];
            let was_extern_driven = if let NodeType::IO { extern_drv, .. } = node.node_type {
                extern_drv
            } else {
                warn!("Node {} is not an IO: {:#?}", node_idx, node);
                return;
            };

            // If the node didn't change states, then ignore this call.
            if was_extern_driven == driven {
                return;
            }

            if let NodeType::IO { extern_drv, .. } = &mut node.node_type {
                *extern_drv = driven;
            }

            (!was_extern_driven, node.edges.clone())
        };

        // Then update neighbors and add them to the edge set if the DC switched 0 <--> !0.
        self.update_edges(edges, driving);
    }

    /// Step the simulation up to `max_node_evals` times. Returns true if more steps are needed.
    pub fn step_simulation(&mut self, max_node_evals: u64) -> bool {
        let stop = self.node_eval_count + max_node_evals;

        while self.edge_set.len() > 0 && self.node_eval_count < stop {
            let node_idx = self.edge_set.pop_front().unwrap();
            let node = self.nodes[node_idx].clone();

            match node.node_type {
                NodeType::IO { .. } => {
                    // TODO: Notify a 'listener' that an I/O was internally driven.
                }
                NodeType::NPN {
                    base_path_idx,
                    collector_path_idx,
                    ..
                } => {
                    let base_dc = self.path_dc[base_path_idx];
                    let collector_dc = self.path_dc[collector_path_idx];
                    let driving = base_dc > 0 && collector_dc > 0;
                    self.update_edges(node.edges, driving);
                }
                NodeType::PNP {
                    base_path_idx,
                    collector_path_idx,
                    ..
                } => {
                    let base_dc = self.path_dc[base_path_idx];
                    let collector_dc = self.path_dc[collector_path_idx];
                    let driving = base_dc == 0 && collector_dc > 0;
                    self.update_edges(node.edges, driving);
                }
            }

            self.node_eval_count += 1;
        }

        self.edge_set.len() > 0
    }

    #[inline(always)]
    pub fn get_path_dc(&self, path_idx: usize) -> u16 {
        self.path_dc[path_idx]
    }

    #[inline(always)]
    fn update_edges(&mut self, edges: SmallVec<[Edge; 2]>, driving: bool) {
        for edge in edges {
            let neighbor = &mut self.nodes[edge.to_node_idx as usize];
            match (edge.edge_type, neighbor.node_type) {
                (
                    EdgeType::Base,
                    NodeType::NPN { base_path_idx, .. } | NodeType::PNP { base_path_idx, .. },
                ) => {
                    if self.drive_dc(base_path_idx, driving) {
                        self.edge_set.push_back(edge.to_node_idx as usize);
                    }
                }
                (
                    EdgeType::Collector,
                    NodeType::NPN {
                        collector_path_idx, ..
                    }
                    | NodeType::PNP {
                        collector_path_idx, ..
                    },
                ) => {
                    if self.drive_dc(collector_path_idx, driving) {
                        self.edge_set.push_back(edge.to_node_idx as usize);
                    }
                }
                (EdgeType::IoOut, NodeType::IO { path_idx, .. }) => {
                    if self.drive_dc(path_idx, driving) {
                        self.edge_set.push_back(edge.to_node_idx as usize);
                    }
                }
                _ => panic!("Invalid graph"),
            }
        }
    }

    #[inline(always)]
    fn drive_dc(&mut self, path_idx: usize, driving: bool) -> bool {
        let dc = &mut self.path_dc[path_idx];
        let new_dc = if driving {
            *dc + 1
        } else {
            debug_assert!(*dc as i32 - 1 >= 0);
            *dc - 1
        };

        let switched = *dc == 0 || new_dc == 0;
        *dc = new_dc;
        switched
    }
}
