use glam::IVec2;

use crate::utils::Dirs;

/// A single Cell in an overall logic-paint IC. This data structure is layed out for easy
/// editing as it matches how a user "draws" on the IC closely. It is not used for simulation
/// however, a `Network` type is generated from a IC instead.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    pub metal: Metal,
    pub si: Silicon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metal {
    None,
    Trace {
        has_via: bool,
        dirs: Dirs,
        path: usize,
    },
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
        dirs: Dirs,
        path: usize,
    },
    Mosfet {
        is_npn: bool,
        gate_dirs: Dirs,
        ec_dirs: Dirs,
        path: usize,
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
