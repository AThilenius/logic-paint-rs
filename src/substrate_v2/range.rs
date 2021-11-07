use super::coords::CellCoord;

/// Range: An abstract "selection" of cell / modules which can be applied to a Buffer to get or set
/// a range of cells at once.

pub enum Range {
    Rectangle {
        lower_left: CellCoord,
        upper_right: CellCoord,
    },
}

impl Range {
    pub fn from_rectangle<TF, TS>(first_point: TF, second_point: TF) -> Self
    where
        TF: Into<CellCoord>,
        TF: Into<CellCoord>,
    {
        let first_point: CellCoord = first_point.into();
        let second_point: CellCoord = second_point.into();
        Self::Rectangle {
            lower_left: CellCoord(first_point.0.min(second_point.0)),
            upper_right: CellCoord(first_point.0.max(second_point.0)),
        }
    }
}
