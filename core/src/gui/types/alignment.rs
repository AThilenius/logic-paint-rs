use serde::{Deserialize, Serialize};

// Which axis is being dynamically layed out. The other axis is either treated as a fixed size or
// an auto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Alignment {
    // Children elements will be layed out horizontally, with the vertical axis will be computed
    // with the skyline algorithm.
    Column,

    // Children elements will be layed out vertically, with the horizontal axis will be computed
    // with the skyline algorithm.
    Row,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Column
    }
}
