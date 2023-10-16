use serde::{Deserialize, Serialize};

use crate::gui::types::{alignment::Alignment, box_size::BoxSize, Len, Position, Rect};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Layout {
    pub alignment: Alignment,
    pub width: Len,
    pub height: Len,
    pub position: Position,
    pub margin: BoxSize,
    pub padding: BoxSize,
    pub rect: Rect,
}
