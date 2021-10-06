use glam::IVec2;
use smallvec::{smallvec, SmallVec};

/// A single Cell in an overall logic-paint IC. This data structure is layed out for easy
/// editing as it matches how a user "draws" on the IC closely. It is not used for simulation
/// however, a `Network` type is generated from a IC instead.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    pub metal: Metal,
    pub si: Silicon,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CellDirs {
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

impl From<IVec2> for CellDirs {
    #[inline(always)]
    fn from(v: IVec2) -> Self {
        Self {
            up: v == IVec2::Y,
            right: v == IVec2::X,
            down: v == -IVec2::Y,
            left: v == -IVec2::X,
        }
    }
}

impl CellDirs {
    #[inline(always)]
    pub fn get_offsets(&self) -> SmallVec<[IVec2; 4]> {
        let mut vec = smallvec![];

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

    pub fn set_direction(&mut self, dir: IVec2, value: bool) {
        // Note y coords are inverted.
        match (dir.x, dir.y) {
            (0, 1) => self.up = value,
            (1, 0) => self.right = value,
            (0, -1) => self.down = value,
            (-1, 0) => self.left = value,
            _ => panic!("Unsupported vector for set_direction"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metal {
    None,
    Trace { has_via: bool, dirs: CellDirs },
    IO { dirs: CellDirs },
}

impl Default for Metal {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Silicon {
    None,
    NP {
        is_n: bool,
        dirs: CellDirs,
    },
    Mosfet {
        is_npn: bool,
        gate_dirs: CellDirs,
        ec_dirs: CellDirs,
    },
}

impl Default for Silicon {
    fn default() -> Self {
        Self::None
    }
}

impl Silicon {
    pub fn matches_n(&self, n: bool) -> bool {
        match self {
            Silicon::NP { is_n, .. } if *is_n == n => true,
            Silicon::Mosfet { is_npn, .. } if *is_npn == n => true,
            _ => false,
        }
    }
}

impl Cell {
    #[inline(always)]
    pub fn pack_into_4_bytes(&self, buf: &mut [u8]) {
        for i in 0..4usize {
            buf[i] = 0;
        }
    }
}
