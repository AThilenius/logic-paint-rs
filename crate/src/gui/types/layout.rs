use serde::{Deserialize, Serialize};

use super::{alignment::Alignment, box_size::BoxSize, Length, Position};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Layout {
    pub alignment: Alignment,
    pub width: Length,
    pub height: Length,
    pub position: Position,
    pub margin: BoxSize,
    pub padding: BoxSize,
}