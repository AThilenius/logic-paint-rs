use std::collections::{HashMap, HashSet};

use glam::IVec2;

use crate::sim::{Graph, Network};

use super::{cell::Cell, Metal};

pub const CHUNK_SIZE: usize = 32;
pub const LOG_CHUNK_SIZE: usize = 5;

#[derive(Default)]
pub struct IntegratedCircuit {
    cell_chunks: HashMap<IVec2, HashMap<IVec2, Cell>>,
    io_cell_locs: HashSet<IVec2>,
    network: Network,
    graph: Graph,
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
            let chunk = if let Some(chunk) = self.cell_chunks.get_mut(&loc) {
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
    }

    pub fn get_chunk(&self, chunk_loc: &IVec2) -> Option<&HashMap<IVec2, Cell>> {
        self.cell_chunks.get(&chunk_loc)
    }

    #[inline(always)]
    pub fn iter_io_cell_locs(&self) -> std::collections::hash_set::Iter<'_, IVec2> {
        self.io_cell_locs.iter()
    }

    pub fn compile(&mut self) {
        self.network = Network::compile_ic(&self);
        self.graph = Graph::from(&self.network);
    }
}

#[inline(always)]
pub fn cell_to_chunk_loc(loc: &IVec2) -> IVec2 {
    // Right shift LOG(CHUNK_SIZE) bits, which is: divide by 32, with a floor op.
    IVec2::new(loc.x >> LOG_CHUNK_SIZE, loc.y >> LOG_CHUNK_SIZE)
}
