use bevy::math::IVec2;
use smallvec::{smallvec, SmallVec};

/// A single Cell in an overall logic-paint canvas. This data structure is layed out for easy
/// editing as it matches how a user "draws" on the canvas closely. It is not used for simulation
/// however, a `Network` type is generated from a canvas instead.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    pub metal: Metal,
    pub si: Silicon,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CellDirs {
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}

impl From<IVec2> for CellDirs {
    #[inline(always)]
    fn from(v: IVec2) -> Self {
        Self {
            up: v == -IVec2::Y,
            right: v == IVec2::X,
            down: v == IVec2::Y,
            left: v == -IVec2::X,
        }
    }
}

impl CellDirs {
    #[inline(always)]
    pub fn get_offsets(&self) -> SmallVec<[IVec2; 4]> {
        let mut vec = smallvec![];

        if self.up {
            vec.push(-IVec2::Y)
        }
        if self.right {
            vec.push(IVec2::X)
        }
        if self.down {
            vec.push(IVec2::Y)
        }
        if self.left {
            vec.push(-IVec2::X)
        }

        vec
    }

    pub fn set_direction(&mut self, dir: IVec2, value: bool) {
        // Note y coords are inverted.
        match (dir.x, dir.y) {
            (0, -1) => self.up = value,
            (1, 0) => self.right = value,
            (0, 1) => self.down = value,
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

        // Bit field masks (3 bytes)
        let si_n = 1u8 << 7;
        let si_p = 1u8 << 6;
        // let si_active = 1u8 << 5;
        let si_dir_up = 1u8 << 4;
        let si_dir_right = 1u8 << 3;
        let si_dir_down = 1u8 << 2;
        let si_dir_left = 1u8 << 1;

        let gate_dir_up = 1u8 << 7;
        let gate_dir_right = 1u8 << 6;
        let gate_dir_down = 1u8 << 5;
        let gate_dir_left = 1u8 << 4;
        // let gate_active = 1u8 << 3;

        let metal = 1u8 << 7;
        let metal_dir_up = 1u8 << 6;
        let metal_dir_right = 1u8 << 5;
        let metal_dir_down = 1u8 << 4;
        let metal_dir_left = 1u8 << 3;
        // let metal_active = 1u8 << 2;
        let via = 1u8 << 1;
        let is_io = 1u8 << 0;

        match self.si {
            Silicon::NP { is_n, dirs, .. }
            | Silicon::Mosfet {
                is_npn: is_n,
                ec_dirs: dirs,
                ..
            } => {
                buf[0] |= if is_n { si_n } else { si_p };
                buf[0] |= if dirs.up { si_dir_up } else { 0 };
                buf[0] |= if dirs.right { si_dir_right } else { 0 };
                buf[0] |= if dirs.down { si_dir_down } else { 0 };
                buf[0] |= if dirs.left { si_dir_left } else { 0 };
            }
            _ => {}
        }

        match self.si {
            // Silicon::NP { is_n, .. } => {
            //     buf[0] |= if is_n { si_n } else { si_p };
            //     // TODO: Si active (1 << 5)
            // }
            Silicon::Mosfet { gate_dirs, .. } => {
                buf[1] |= if gate_dirs.up { gate_dir_up } else { 0 };
                buf[1] |= if gate_dirs.right { gate_dir_right } else { 0 };
                buf[1] |= if gate_dirs.down { gate_dir_down } else { 0 };
                buf[1] |= if gate_dirs.left { gate_dir_left } else { 0 };

                // TODO: Gate/EC active
            }
            _ => {}
        }

        match self.metal {
            Metal::IO { dirs } | Metal::Trace { dirs, .. } => {
                buf[2] |= metal;
                buf[2] |= if dirs.up { metal_dir_up } else { 0 };
                buf[2] |= if dirs.right { metal_dir_right } else { 0 };
                buf[2] |= if dirs.down { metal_dir_down } else { 0 };
                buf[2] |= if dirs.left { metal_dir_left } else { 0 };

                // TODO: Metal active
            }
            Metal::None => {}
        }

        match self.metal {
            Metal::IO { .. } => {
                buf[2] |= is_io;
            }
            Metal::Trace { has_via: true, .. } => {
                buf[2] |= via;
            }
            _ => {}
        }
    }
}
