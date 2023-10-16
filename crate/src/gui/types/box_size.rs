use serde::{Deserialize, Serialize};

use crate::gui::types::{point::Point, Size};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum BoxSize {
    Uniform(f32),
    NonUniform {
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
}

impl Default for BoxSize {
    fn default() -> Self {
        Self::Uniform(0.0)
    }
}

#[allow(unused)]
impl BoxSize {
    pub const ZERO: BoxSize = BoxSize::Uniform(0.0);
    pub const ONE: BoxSize = BoxSize::Uniform(1.0);

    pub fn sum(&self) -> Size {
        Size {
            width: self.left() + self.right(),
            height: self.top() + self.bottom(),
        }
    }

    pub fn top_left(&self) -> Point {
        Point {
            top: self.top(),
            left: self.left(),
        }
    }

    pub fn left(&self) -> f32 {
        match self {
            Self::Uniform(v) => *v,
            Self::NonUniform { left, .. } => *left,
        }
    }

    pub fn top(&self) -> f32 {
        match self {
            Self::Uniform(v) => *v,
            Self::NonUniform { top, .. } => *top,
        }
    }

    pub fn right(&self) -> f32 {
        match self {
            Self::Uniform(v) => *v,
            Self::NonUniform { right, .. } => *right,
        }
    }

    pub fn bottom(&self) -> f32 {
        match self {
            Self::Uniform(v) => *v,
            Self::NonUniform { bottom, .. } => *bottom,
        }
    }
}
