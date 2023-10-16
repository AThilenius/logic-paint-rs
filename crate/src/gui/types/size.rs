use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };
}

impl std::iter::Sum for Size {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, size| acc + size)
    }
}

impl std::ops::Add<Size> for Size {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl std::ops::Sub<Size> for Size {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
