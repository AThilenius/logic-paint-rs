// use std::collections::HashMap;

// use bevy::math::IVec2;
// use petgraph::{
//     dot::{Config, Dot},
//     prelude::*,
// };

// use crate::{
//     canvas::{Cell, Conductor, ConductorLevel},
//     utils::{HilbertCode, HilbertIndexing},
// };

// use super::Canvas;

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// struct Node {
//     /// NodeID and hilbert code for original cell.
//     code: HilbertCode,
//     t: NodeType,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum NodeType {
//     ERR,
//     IO { driving_high: bool },
//     NPN {},
//     PNP {},
// }

// impl Default for NodeType {
//     fn default() -> Self {
//         NodeType::ERR
//     }
// }

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// struct Edge {
//     /// The network id this edge belongs to.
//     nid: u32,

//     /// The type of thing we are driving (edges are one-way, remember).
//     t: DriveType,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum DriveType {
//     ERR,
//     Base,
//     IO,
//     EC,
// }

// impl Default for DriveType {
//     fn default() -> Self {
//         DriveType::ERR
//     }
// }

// impl Into<NodeIndex> for Node {
//     fn into(self) -> NodeIndex {
//         NodeIndex::new(self.code.into())
//     }
// }

// pub fn compile_cavas_data(data: &Canvas) {
//     println!("compile_cavas_data...");
//     // Cells already explored, and optionally the NodeIndex at the explored cell.
//     let mut explored_cells: HashMap<IVec2, Option<NodeIndex>> = HashMap::new();

//     // The backward link between a graph edge and the cells it was generated from. Used for
//     // rendering.
//     let mut netlist: Vec<Vec<Conductor>> = vec![];

//     // The overall BFS edge set, as a tuple of (the Node was generated this edge, conductor to
//     // explore off). That means that a IO, NPN or PNP cell will never be added to this set, they are
//     // implicitly traversed.
//     let mut edge_set: Vec<Conductor> = vec![];

//     // The graph we will be constructing.
//     let mut g = Graph::<Node, Edge>::new();

//     // Prime the graph and edge_set but generating nodes for each I/O pin and adding all it's
//     // connected neighbors to the edge_set.
//     for io in data.io_pins.values() {
//         let cell = data.cells.get(io.loc);
//         println!("IO Pin at {}", io.loc);

//         let node_idx = g.add_node(Node {
//             code: io.loc.into(),
//             t: NodeType::IO {
//                 driving_high: io.is_constant,
//             },
//         });

//         explored_cells.insert(io.loc, Some(node_idx));

//         // IO pins can only be connected via metal, so that's all we need to iterate.
//         for offset in cell.metal_dirs.get_offsets() {
//             println!(
//                 "Adding initial IO pin neighbor: {} + {} = {}",
//                 io.loc,
//                 offset,
//                 io.loc + offset
//             );
//             edge_set.push(Conductor {
//                 loc: io.loc + offset,
//                 level: ConductorLevel::Metal,
//             });
//         }
//     }

//     // Open the explore set
//     //   - Start a new netlist
//     //     - Push a new vector onto the netlists vector
//     //     - Outer vector index is out nid, inner vector stores all cells belonging to the network
//     //   - Add metal, vias, same-type silicon (bredth first) to netlist
//     //   - Add other attached BASE or E/C nodes attached to the NPN/PNP to the explore set if not
//     //     alrady marked explored.
//     //   - Loop until explore set is empty
//     while edge_set.len() > 0 {
//         let c = edge_set.pop().unwrap();
//         let nid = netlist.len();
//         println!("============================================================");
//         println!("Beginning edge explore at {}", c.loc);

//         // Bredth-first search the conductor. Find standrad connections, and connectes to edges. The
//         // former is added to the netlist, the latter is used to get/construct graph nodes and add
//         // graph edges.
//         let (conductors, conductor_edges) = data.explore_conductor(c);

//         println!("Traced conductors:");
//         for c in conductors.iter() {
//             println!("  - {:?}", c);
//         }
//         println!("Traced conductor edge:");
//         for c in conductor_edges.iter() {
//             println!("  - {:?}", c);
//         }

//         // Add normal conductors to the explored set directly.
//         explored_cells.extend(conductors.iter().map(|c| (c.loc, None)));

//         // - Open IO/NPN/PNP edges and convert it into tuples of (Conductor, Cell, NodeIndex).
//         let conductor_edges: Vec<(Conductor, Cell, NodeIndex)> = conductor_edges
//             .iter()
//             .map(|conductor_edge| {
//                 let cell = data.cells.get(conductor_edge.loc);

//                 // Get or create the graph node and node index.
//                 let idx = if let Some(Some(node_idx)) = explored_cells.get(&conductor_edge.loc) {
//                     *node_idx
//                 } else {
//                     g.add_node(Node {
//                         code: conductor_edge.loc.into(),
//                         t: match conductor_edge.level {
//                             ConductorLevel::Gate if cell.si_p => NodeType::NPN {},
//                             ConductorLevel::Gate if cell.si_n => NodeType::PNP {},
//                             ConductorLevel::Metal if cell.is_io => {
//                                 let io = data.io_pins.get(&conductor_edge.loc).unwrap();
//                                 NodeType::IO {
//                                     driving_high: io.is_constant,
//                                 }
//                             }
//                             _ => panic!(
//                                 "Edge in conductor edge set is an invalid graph node {:?}",
//                                 conductor_edge
//                             ),
//                         },
//                     })
//                 };

//                 // Go ahead and add the edge node to the explored set because now is a convenient time
//                 // to do so.
//                 explored_cells.insert(conductor_edge.loc, Some(idx));

//                 (*conductor_edge, *cell, idx)
//             })
//             .collect();

//         // Filter the edges into a few categories.
//         let gate_nodes = conductor_edges
//             .iter()
//             .filter(|(_, cell, _)| !cell.gate_dirs.is_none());

//         let ec_nodes = conductor_edges
//             .iter()
//             .filter(|(_, cell, _)| cell.gate_dirs.is_none() && !cell.is_io);

//         let io_nodes = conductor_edges.iter().filter(|(_, cell, _)| cell.is_io);

//         println!("Of those, they breakdown to:");
//         println!("  - gate_nodes:");
//         for (c, cell, idx) in gate_nodes.clone() {
//             println!("    - {} at level {:?}", c.loc, c.level);
//         }

//         println!("  - ec_nodes:");
//         for (c, cell, idx) in ec_nodes.clone() {
//             println!("    - {} at level {:?}", c.loc, c.level);
//         }

//         println!("  - io_nodes:");
//         for (c, cell, idx) in io_nodes.clone() {
//             println!("    - {} at level {:?}", c.loc, c.level);
//         }

//         // TODO: I need to add parts of the conductor_edges to this, but not the SI under a gate.
//         netlist.push(conductors);

//         for (_, _, from_idx) in ec_nodes.clone().chain(io_nodes.clone()) {
//             // Add a (BASE, nid) edge from all [E/C, IO] nodes to all Base nodes (one way)
//             for (_, _, to_idx) in gate_nodes.clone() {
//                 g.add_edge(
//                     *from_idx,
//                     *to_idx,
//                     Edge {
//                         nid: nid as u32,
//                         t: DriveType::Base,
//                     },
//                 );
//             }

//             // Add an (E/C, nid) edge from all [E/C, IO] nodes to all other E/C nodes (two way)
//             for (_, _, to_idx) in ec_nodes.clone() {
//                 g.add_edge(
//                     *from_idx,
//                     *to_idx,
//                     Edge {
//                         nid: nid as u32,
//                         t: DriveType::EC,
//                     },
//                 );
//             }
//         }

//         for (_, _, from_idx) in ec_nodes.clone() {
//             // Add a (DRV, nid) edge from all E/C nodes to all IO nodes (one way)
//             for (_, _, to_idx) in io_nodes.clone() {
//                 g.add_edge(
//                     *from_idx,
//                     *to_idx,
//                     Edge {
//                         nid: nid as u32,
//                         t: DriveType::IO,
//                     },
//                 );
//             }
//         }

//         // TODO: This part be broke.
//         // Now explore all neightbors of the conductor edges, they still need to be searched.
//         // However, only MOSFETS need to be handled as IO neightbors were already added above.
//         for (c, cell, _) in conductor_edges {
//             println!("Checking explore off of cell {} level {:?}", c.loc, c.level);
//             if c.level == ConductorLevel::Gate || c.level == ConductorLevel::Si {
//                 // Add the cells connected via both the gate and the Si under the gate if they
//                 // aren't already explored.
//                 for offset in cell.gate_dirs.get_offsets() {
//                     if !explored_cells.contains_key(&(c.loc + offset)) {
//                         edge_set.push(Conductor {
//                             loc: c.loc + offset,
//                             level: ConductorLevel::Si,
//                         });
//                     }
//                 }

//                 for offset in cell.si_dirs.get_offsets() {
//                     if !explored_cells.contains_key(&(c.loc + offset)) {
//                         edge_set.push(Conductor {
//                             loc: c.loc + offset,
//                             level: ConductorLevel::Si,
//                         });
//                     }
//                 }
//             }
//         }
//     }

//     println!("{:?}", Dot::with_config(&g, &[Config::EdgeNoLabel]));
// }

// // Nodes:
// // A - I/O - Constant 1
// // B - I/O & TR1:B
// // C - I/O & TR2:B & Q

// // Edges (directed):
// // A -- (E/C) --> B
// // B -- (E/C) --> C
// // C -- (BAS) --> C
