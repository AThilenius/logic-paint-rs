use std::{collections::HashMap, fmt};

use smallvec::SmallVec;

use crate::sim::AtomType;

use super::Network;

/// A directed, cyclic graph that represents only the logic needed to simulate I/O, NPN and PNP
/// logic, with a look-aside buffer for translating that simulation back into Paths.
///
/// Paths get boiled down into Nodes and Edges, where a node is one of three types: I/O, NPN or PNP
/// and edges define what is being DRIVEN on the node: IoOut, Base, Collector. A single Path (all the
/// atoms connector by a conductor) is boiled down into these directed edges. For example, if a
/// single I/O is connected to two gates, then it will have two edges, one to each gate. This is
/// optimal for small Paths (which is more of the graph) but a degenerate for large shared busses.
/// Each input on a node stores a count of how many things are "driving" it high. When these counts
/// go to or from 0, the node is reevaluated for a state change. If it's state changed, the
/// downstream edges are added to a Breadth First Queue which will correctly propagate transistor
/// delay through the circuit.
///
/// An out of band index of edges to source path indexes is kept for rendering. It is not used at all
/// during simulation however.
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edge_to_path_look_aside: Vec<SmallVec<[u32; 4]>>,
}

impl From<Network> for Graph {
    fn from(network: Network) -> Self {
        // Filter all terminal atoms
        let terminal_atoms = network
            .paths
            .iter()
            .flat_map(|p| p.atoms.clone())
            .filter(|a| a.atom_type != AtomType::NonMetal && a.atom_type != AtomType::Metal);

        // Map each terminal atom's location (not the atom itself) to Node in the graph.
        let mut nodes = Vec::new();
        let mut term_to_node = HashMap::new();
        for terminal_atom in terminal_atoms {
            let loc = terminal_atom.src_loc;
            if term_to_node.contains_key(&loc) {
                continue;
            }

            match terminal_atom.atom_type {
                AtomType::TerminalIoPin { .. } => {
                    term_to_node.insert(loc, nodes.len());
                    nodes.push(Node {
                        edges: Default::default(),
                        node_type: NodeType::IO {
                            extern_drv: false,
                            dc: 0,
                        },
                    });
                }
                AtomType::TerminalMosfetEC { is_npn: true, .. }
                | AtomType::TerminalMosfetBase { is_npn: true, .. } => {
                    term_to_node.insert(loc, nodes.len());
                    nodes.push(Node {
                        edges: Default::default(),
                        node_type: NodeType::NPN {
                            base_dc: 0,
                            collector_dc: 0,
                        },
                    });
                }
                AtomType::TerminalMosfetEC { is_npn: false, .. }
                | AtomType::TerminalMosfetBase { is_npn: false, .. } => {
                    term_to_node.insert(loc, nodes.len());
                    nodes.push(Node {
                        edges: Default::default(),
                        node_type: NodeType::PNP {
                            base_dc: 0,
                            collector_dc: 0,
                        },
                    });
                }
                _ => {}
            }
        }

        // Now we can enumerate paths and build up edges between nodes. We store a look-aside buffer
        // of edges to source path_idx.
        let mut edge_to_path_look_aside: Vec<SmallVec<[u32; 4]>> =
            vec![Default::default(); nodes.len()];
        for (i, path) in network.paths.iter().enumerate() {
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
                let from_node_idx = *term_to_node.get(&from_atom.src_loc).unwrap();

                // Add a Base edge from all [E/C, IO] nodes to all Base nodes (one way)
                for to_atom in gate_nodes.clone() {
                    let to_node_idx = *term_to_node.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::Base,
                    });
                    edge_to_path_look_aside[from_node_idx].push(i as u32);
                }

                // Add a Collector edge from all [E/C, IO] nodes to all other E/C nodes (two way)
                for to_atom in ec_nodes.clone() {
                    if from_atom == to_atom {
                        continue;
                    }

                    let to_node_idx = *term_to_node.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::Collector,
                    });
                    edge_to_path_look_aside[from_node_idx].push(i as u32);
                }
            }

            for from_atom in ec_nodes.clone() {
                let from_node_idx = *term_to_node.get(&from_atom.src_loc).unwrap();

                // Add an IoOut edge from all E/C nodes to all IO nodes (one way)
                for to_atom in io_nodes.clone() {
                    let to_node_idx = *term_to_node.get(&to_atom.src_loc).unwrap();
                    nodes.get_mut(from_node_idx).unwrap().edges.push(Edge {
                        to_node_idx: to_node_idx as u32,
                        edge_type: EdgeType::IoOut,
                    });
                    edge_to_path_look_aside[from_node_idx].push(i as u32);
                }
            }
        }

        Graph {
            nodes,
            edge_to_path_look_aside,
        }
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Graph {{")?;
        for (i, node) in self.nodes.iter().enumerate() {
            write!(f, "  {:02} Node ", i)?;
            match node.node_type {
                NodeType::IO { extern_drv, dc } => {
                    write!(f, "IO extern:{}, dc:{}", extern_drv, dc)?
                }
                NodeType::NPN {
                    base_dc,
                    collector_dc,
                } => write!(f, "NPN base_dc:{}, collector_dc:{}", base_dc, collector_dc)?,
                NodeType::PNP {
                    base_dc,
                    collector_dc,
                } => write!(f, "PNP base_dc:{}, collector_dc:{}", base_dc, collector_dc)?,
            }
            writeln!(f, " {{")?;

            for edge in node.edges.iter() {
                writeln!(
                    f,
                    "    self -> {:02}: {}",
                    edge.to_node_idx,
                    match edge.edge_type {
                        EdgeType::IoOut => "IoOut",
                        EdgeType::Base => "Base",
                        EdgeType::Collector => "Collector",
                    }
                )?;
            }

            writeln!(f, "  }}")?;
        }

        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    node_type: NodeType,
    edges: SmallVec<[Edge; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    IO { extern_drv: bool, dc: u16 },
    NPN { base_dc: u16, collector_dc: u16 },
    PNP { base_dc: u16, collector_dc: u16 },
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
