use serde::{Deserialize, Serialize};

use super::{point::Point, Size};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct BoxSize {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[allow(unused)]
impl BoxSize {
    pub const ZERO: BoxSize = BoxSize::equal(0.0);
    pub const ONE: BoxSize = BoxSize::equal(1.0);

    pub const fn equal(size: f32) -> Self {
        Self {
            left: size,
            top: size,
            right: size,
            bottom: size,
        }
    }

    pub fn sum(&self) -> Size {
        Size {
            width: self.left + self.right,
            height: self.top + self.bottom,
        }
    }

    pub fn top_left(&self) -> Point {
        Point {
            top: self.top,
            left: self.left,
        }
    }
}
