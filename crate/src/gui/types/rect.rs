use serde::{Deserialize, Serialize};

use super::{box_size::BoxSize, point::Point, Size};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl std::ops::Sub<BoxSize> for Rect {
    type Output = Self;

    fn sub(self, rhs: BoxSize) -> Self::Output {
        Self {
            origin: Point {
                top: self.origin.top + rhs.top,
                left: self.origin.left + rhs.left,
            },
            size: Size {
                width: self.size.width - rhs.left - rhs.right,
                height: self.size.height - rhs.top - rhs.bottom,
            },
        }
    }
}
