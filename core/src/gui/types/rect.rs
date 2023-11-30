use serde::{Deserialize, Serialize};

use crate::gui::types::{box_size::BoxSize, point::Point, Size};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}
impl Rect {
    pub fn contains(&self, point: Point) -> bool {
        point.top >= self.origin.top
            && point.top <= self.origin.top + self.size.height
            && point.left >= self.origin.left
            && point.left <= self.origin.left + self.size.width
    }
}

impl std::ops::Sub<BoxSize> for Rect {
    type Output = Self;

    fn sub(self, rhs: BoxSize) -> Self::Output {
        Self {
            origin: Point {
                top: self.origin.top + rhs.top(),
                left: self.origin.left + rhs.left(),
            },
            size: Size {
                width: self.size.width - rhs.left() - rhs.right(),
                height: self.size.height - rhs.top() - rhs.bottom(),
            },
        }
    }
}
