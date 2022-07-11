use glam::IVec2;

use crate::coords::CellCoord;

#[derive(Clone)]
pub struct Selection {
    // The lower left point in the selection, inclusively.
    pub lower_left: CellCoord,

    // The lower left point in the selection, exclusively (ie row/col not included).
    pub upper_right: CellCoord,
}

impl Selection {
    pub fn from_rectangle_inclusive<TF, TS>(first_point: TF, second_point: TS) -> Self
    where
        TF: Into<CellCoord>,
        TS: Into<CellCoord>,
    {
        let first_point: CellCoord = first_point.into();
        let second_point: CellCoord = second_point.into();

        let lower_left = CellCoord(first_point.0.min(second_point.0));
        let upper_right = CellCoord(first_point.0.max(second_point.0) + IVec2::ONE);

        Self {
            lower_left,
            upper_right,
        }
    }

    #[inline(always)]
    pub fn test<T>(&self, point: T) -> bool
    where
        T: Into<CellCoord>,
    {
        let cell_coord: CellCoord = point.into();

        cell_coord.0.x >= self.lower_left.0.x
            && cell_coord.0.y >= self.lower_left.0.y
            && cell_coord.0.x < self.upper_right.0.x
            && cell_coord.0.y < self.upper_right.0.y
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            lower_left: (0, 0).into(),
            upper_right: (0, 0).into(),
        }
    }
}
