use std::collections::{HashMap, HashSet};

use glam::IVec2;

use crate::{
    log,
    sim::{Atom, AtomType, Graph, Path},
};

use super::{cell::Cell, Metal, Silicon};

pub const CHUNK_SIZE: usize = 32;
pub const LOG_CHUNK_SIZE: usize = 5;

pub struct IntegratedCircuit {
    cell_chunks: HashMap<IVec2, HashMap<IVec2, Cell>>,
    io_cell_locs: HashSet<IVec2>,
    dirty: bool,
    compiled_paths: Option<Vec<Path>>,
    graph: Option<Graph>,
}

impl Default for IntegratedCircuit {
    fn default() -> Self {
        Self {
            cell_chunks: Default::default(),
            io_cell_locs: Default::default(),
            dirty: true,
            compiled_paths: None,
            graph: None,
        }
    }
}

impl IntegratedCircuit {
    pub fn get_cell(&self, loc: &IVec2) -> Option<Cell> {
        if let Some(chunk) = self.cell_chunks.get(&cell_to_chunk_loc(loc)) {
            chunk.get(loc).map(Clone::clone)
        } else {
            None
        }
    }

    pub fn set_cell(&mut self, loc: IVec2, cell: Cell) {
        let chunk_loc = cell_to_chunk_loc(&loc);
        if cell == Default::default() {
            // Delete op.
            let mut last_cell = false;
            if let Some(chunk) = self.cell_chunks.get_mut(&chunk_loc) {
                chunk.remove(&loc);
                if chunk.len() == 0 {
                    last_cell = true;
                }
            }
            if last_cell {
                self.cell_chunks.remove(&chunk_loc);
            }
            self.io_cell_locs.remove(&loc);
        } else {
            // Set op. Need to create a chunk if it doesn't already exist.
            let chunk = if let Some(chunk) = self.cell_chunks.get_mut(&chunk_loc) {
                chunk
            } else {
                self.cell_chunks
                    .insert(chunk_loc.clone(), Default::default());
                self.cell_chunks.get_mut(&chunk_loc).unwrap()
            };
            chunk.insert(loc, cell.clone());

            if let Metal::IO { .. } = cell.metal {
                self.io_cell_locs.insert(loc);
            }
        }

        self.set_dirty();
    }

    pub fn get_chunk(&self, chunk_loc: &IVec2) -> Option<&HashMap<IVec2, Cell>> {
        self.cell_chunks.get(&chunk_loc)
    }

    #[inline(always)]
    pub fn get_path_dc(&self, path_idx: usize) -> u16 {
        if let Some(graph) = &self.graph {
            graph.get_path_dc(path_idx)
        } else {
            0_u16
        }
    }

    pub fn compile_or_get_graph_mut(&mut self) -> &mut Graph {
        if self.dirty {
            log!("IC beginning compilation");
            self.dirty = false;

            if self.compiled_paths.is_none() {
                log!("Compiling paths...");
                self.compile_paths();
                log!("Done");
                log!("Compiling graph...");
                self.graph = Some(Graph::new(self.compiled_paths.as_ref().unwrap()));
                log!("Done.");
            }
        }

        self.graph.as_mut().unwrap()
    }

    fn compile_paths(&mut self) {
        let mut paths = Vec::new();
        let mut explored: HashSet<(IVec2, AtomType)> = HashSet::new();

        // Seed the edge set with IO pin terminal atoms.
        let mut edge_set: Vec<Atom> = self
            .io_cell_locs
            .iter()
            .map(|l| Atom {
                src_loc: *l,
                path: usize::MAX,
                atom_type: AtomType::TerminalIoPin,
            })
            .collect();

        // Breadth-first search of all paths that connect to at least one IO pin.
        while edge_set.len() > 0 {
            let mut atom = edge_set.pop().unwrap();
            if explored.contains(&(atom.src_loc, atom.atom_type)) {
                continue;
            }

            log!("Exploring atom: {:#?}", atom);

            // Assign the atom to this path (needed because the seed atoms aren't assigned)
            atom.path = paths.len();

            let path =
                Path::explore_atom_and_update_cell_paths(self, &mut explored, atom, paths.len());

            // Collect all terminal atoms from the path and add connecting MOSFET atoms to the
            // explore set.
            for atom in path.atoms.iter() {
                let cell = self.get_cell(&atom.src_loc).unwrap();
                match (atom.atom_type, cell.si) {
                    (AtomType::TerminalMosfetBase { is_npn }, Silicon::Mosfet { ec_dirs, .. }) => {
                        // Add both Emitter/Collector atoms.
                        for offset in ec_dirs.get_offsets() {
                            edge_set.push(Atom {
                                src_loc: atom.src_loc,
                                path: paths.len(),
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
                                    path: paths.len(),
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

            paths.push(path);
        }

        self.compiled_paths = Some(paths);
    }

    fn set_dirty(&mut self) {
        self.dirty = true;
        self.compiled_paths = None;
        self.graph = None;
    }
}

#[inline(always)]
pub fn cell_to_chunk_loc(loc: &IVec2) -> IVec2 {
    // Right shift LOG(CHUNK_SIZE) bits, which is: divide by 32, with a floor op.
    IVec2::new(loc.x >> LOG_CHUNK_SIZE, loc.y >> LOG_CHUNK_SIZE)
}
