use serde::{Deserialize, Serialize};

use super::point::Point;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Position {
    // The position is determined by the layout algorithm. This is analogous to flex-box layout in
    // CSS.
    Standard,

    // The position is fixed in viewport coordinates, starting a new, fresh layout tree. This is
    // much like fixed positioning in CSS.
    Fixed(Point),

    // The position is relative to where it would normally be placed. The element contributes to the
    // layout, but any gaps left are not filled.
    Relative(Point),
}

impl Default for Position {
    fn default() -> Self {
        Self::Standard
    }
}
