use std::collections::{HashMap, HashSet};

use glam::IVec2;
use itertools::Itertools;

use crate::coords::{CellCoord, ChunkCoord};

/// Represents a selection of cells against an arbitrary substrate.

#[derive(Clone)]
pub struct Selection {
    pub cells: HashSet<CellCoord>,
}

impl Selection {
    pub fn from_rectangle<TF, TS>(first_point: TF, second_point: TS) -> Self
    where
        TF: Into<CellCoord>,
        TS: Into<CellCoord>,
    {
        let first_point: CellCoord = first_point.into();
        let second_point: CellCoord = second_point.into();

        let ll = first_point.0.min(second_point.0);
        let ur = first_point.0.max(second_point.0) + IVec2::new(1, 1);

        Self {
            cells: (ll.y..ur.y)
                .flat_map(move |y| (ll.x..ur.x).map(move |x| (x, y).into()))
                .collect(),
        }
    }

    pub fn group_changes_by_chunk(&self) -> Vec<(ChunkCoord, Vec<CellCoord>)> {
        let mut chunks = HashMap::new();

        for cell_coord in &self.cells {
            let chunk_coord: ChunkCoord = cell_coord.into();

            let chunk_vec = &mut chunks.entry(chunk_coord).or_insert_with(|| vec![]);
            chunk_vec.push(*cell_coord);
        }

        chunks.drain().collect_vec()
    }

    pub fn union(&self, rhs: &Selection) -> Selection {
        Self {
            cells: self.cells.union(&rhs.cells).cloned().collect(),
        }
    }

    pub fn difference(&self, rhs: &Selection) -> Selection {
        Self {
            cells: self.cells.difference(&rhs.cells).cloned().collect(),
        }
    }
}
