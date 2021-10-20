use std::ops;

use arrayvec::ArrayVec;
use glam::IVec2;

/// Represents the various placements of Metal and Si within a Cell, including the 4 cardinal
/// directions, and the center "self" location (which is implicit when any cardinal direction is
/// set, but can also stand alone).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Placement {
    pub center: bool,
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

impl Placement {
    pub const NONE: Self = Self {
        center: false,
        up: false,
        right: false,
        down: false,
        left: false,
    };

    pub const CENTER: Self = Self {
        center: true,
        up: false,
        right: false,
        down: false,
        left: false,
    };

    pub const UP: Self = Self {
        center: true,
        up: true,
        right: false,
        down: false,
        left: false,
    };

    pub const RIGHT: Self = Self {
        center: true,
        right: true,
        up: false,
        down: false,
        left: false,
    };

    pub const DOWN: Self = Self {
        center: true,
        down: true,
        up: false,
        right: false,
        left: false,
    };

    pub const LEFT: Self = Self {
        center: true,
        left: true,
        up: false,
        right: false,
        down: false,
    };

    pub fn from_cardinal(dir: IVec2) -> Self {
        match (dir.x, dir.y) {
            (0, 1) => Self::UP,
            (1, 0) => Self::RIGHT,
            (0, -1) => Self::DOWN,
            (-1, 0) => Self::LEFT,
            _ => panic!("Non-unit cardinal direction vector"),
        }
    }

    pub fn tangent(&self) -> Self {
        debug_assert!(*self != Self::NONE && *self != Self::CENTER);

        if self.up || self.down {
            Self::LEFT | Self::RIGHT
        } else {
            Self::UP | Self::DOWN
        }
    }

    pub fn cardinal_vectors(&self) -> ArrayVec<IVec2, 4> {
        let mut vec = ArrayVec::<_, 4>::new();

        if self.up {
            vec.push(IVec2::Y)
        }
        if self.right {
            vec.push(IVec2::X)
        }
        if self.down {
            vec.push(-IVec2::Y)
        }
        if self.left {
            vec.push(-IVec2::X)
        }

        vec
    }

    pub fn set_cardinal(&mut self, dir: IVec2) {
        match (dir.x, dir.y) {
            (0, 1) => self.up = true,
            (1, 0) => self.right = true,
            (0, -1) => self.down = true,
            (-1, 0) => self.left = true,
            _ => panic!("Non-unit cardinal direction vector"),
        }
        self.center = true;
    }

    pub fn has_cardinal(&self, dir: IVec2) -> bool {
        match (dir.x, dir.y) {
            (0, 1) => self.up,
            (1, 0) => self.right,
            (0, -1) => self.down,
            (-1, 0) => self.left,
            _ => panic!("Non-unit cardinal direction vector"),
        }
    }
}

impl ops::BitOr for Placement {
    type Output = Placement;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Output {
            center: self.center | rhs.center,
            left: self.left | rhs.left,
            up: self.up | rhs.up,
            right: self.right | rhs.right,
            down: self.down | rhs.down,
        }
    }
}
