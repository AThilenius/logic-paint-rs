use std::{
    collections::HashSet,
    ops::{Add, Sub},
};

use crate::coords::CellCoord;

/// Represents a selection of cells against an arbitrary substrate.

pub struct Selection {
    pub cells: HashSet<CellCoord>,
}

impl Selection {
    pub fn from_rectangle<TF, TS>(first_point: TF, second_point: TF) -> Self
    where
        TF: Into<CellCoord>,
        TF: Into<CellCoord>,
    {
        let first_point: CellCoord = first_point.into();
        let second_point: CellCoord = second_point.into();

        let ll = first_point.0.min(second_point.0);
        let ur = first_point.0.max(second_point.0);

        Self {
            cells: (ll.y..ur.y)
                .flat_map(move |y| (ll.x..ur.x).map(move |x| (x, y).into()))
                .collect(),
        }
    }
}

impl Add for Selection {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            cells: self.cells.union(&rhs.cells).cloned().collect(),
        }
    }
}

impl Sub for Selection {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            cells: self.cells.difference(&rhs.cells).cloned().collect(),
        }
    }
}
