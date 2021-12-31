use super::coords::CellCoord;

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

    pub fn iter_cell_coords(&self) -> impl Iterator<Item = CellCoord> {
        match self {
            Range::Rectangle {
                lower_left,
                upper_right,
            } => {
                let ll = lower_left.clone();
                let ur = upper_right.clone();
                (ll.0.y..ur.0.y).flat_map(move |y| (ll.0.x..ur.0.x).map(move |x| (x, y).into()))
            }
        }
    }
}
