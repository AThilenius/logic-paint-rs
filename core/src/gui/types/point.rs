use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Point {
    pub top: f32,
    pub left: f32,
}

impl std::ops::Add<Point> for Point {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        Self {
            top: self.top + rhs.top,
            left: self.left + rhs.left,
        }
    }
}
