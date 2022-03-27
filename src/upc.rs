use std::ops;

use arrayvec::ArrayVec;
use glam::IVec2;

pub const UPC_BYTE_LEN: usize = 4;
pub const LOG_UPC_BYTE_LEN: usize = 2;

/// Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for direct blitting
/// to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian agnosticism during blitting.
/// Does not encode BufferMask data. The first 16 bits are also encoded as part of Blueprint
/// serialization.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct UPC(pub [u8; UPC_BYTE_LEN]);

impl UPC {
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0_u8; UPC_BYTE_LEN];
        bytes.copy_from_slice(slice);
        Self(bytes)
    }

    #[inline(always)]
    pub fn get_bit(&self, bit: Bit) -> bool {
        Bit::get(*self, bit)
    }

    #[inline(always)]
    pub fn set_bit(&mut self, bit: Bit) {
        Bit::set(self, bit, true);
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, bit: Bit) {
        Bit::set(self, bit, false);
    }

    pub fn is_mosfet(&self) -> bool {
        self.get_bit(Bit::GATE_DIR_UP)
            | self.get_bit(Bit::GATE_DIR_RIGHT)
            | self.get_bit(Bit::GATE_DIR_DOWN)
            | self.get_bit(Bit::GATE_DIR_LEFT)
    }
}

#[allow(non_camel_case_types)]
pub enum Bit {
    SI_N,
    SI_P,
    SI_DIR_UP,
    SI_DIR_RIGHT,
    SI_DIR_DOWN,
    SI_DIR_LEFT,
    GATE_DIR_UP,
    GATE_DIR_RIGHT,
    GATE_DIR_DOWN,
    GATE_DIR_LEFT,
    METAL,
    METAL_DIR_UP,
    METAL_DIR_RIGHT,
    METAL_DIR_DOWN,
    METAL_DIR_LEFT,
    VIA,
    IO,
}

impl Bit {
    #[inline(always)]
    pub fn get(upc: UPC, bit: Bit) -> bool {
        let upc = upc.0;
        match bit {
            Bit::SI_N => upc[0] & (1 << 7) > 0,
            Bit::SI_P => upc[0] & (1 << 6) > 0,
            Bit::SI_DIR_UP => upc[0] & (1 << 5) > 0,
            Bit::SI_DIR_RIGHT => upc[0] & (1 << 4) > 0,
            Bit::SI_DIR_DOWN => upc[0] & (1 << 3) > 0,
            Bit::SI_DIR_LEFT => upc[0] & (1 << 2) > 0,
            Bit::GATE_DIR_UP => upc[0] & (1 << 1) > 0,
            Bit::GATE_DIR_RIGHT => upc[0] & (1 << 0) > 0,
            Bit::GATE_DIR_DOWN => upc[1] & (1 << 7) > 0,
            Bit::GATE_DIR_LEFT => upc[1] & (1 << 6) > 0,
            Bit::METAL => upc[1] & (1 << 5) > 0,
            Bit::METAL_DIR_UP => upc[1] & (1 << 4) > 0,
            Bit::METAL_DIR_RIGHT => upc[1] & (1 << 3) > 0,
            Bit::METAL_DIR_DOWN => upc[1] & (1 << 2) > 0,
            Bit::METAL_DIR_LEFT => upc[1] & (1 << 1) > 0,
            Bit::VIA => upc[1] & (1 << 0) > 0,
            Bit::IO => upc[2] & (1 << 7) > 0,
        }
    }

    #[inline(always)]
    pub fn set(upc: &mut UPC, bit: Bit, value: bool) {
        let upc = &mut upc.0;
        if value {
            match bit {
                Bit::SI_N => upc[0] |= 1 << 7,
                Bit::SI_P => upc[0] |= 1 << 6,
                Bit::SI_DIR_UP => upc[0] |= 1 << 5,
                Bit::SI_DIR_RIGHT => upc[0] |= 1 << 4,
                Bit::SI_DIR_DOWN => upc[0] |= 1 << 3,
                Bit::SI_DIR_LEFT => upc[0] |= 1 << 2,
                Bit::GATE_DIR_UP => upc[0] |= 1 << 1,
                Bit::GATE_DIR_RIGHT => upc[0] |= 1 << 0,
                Bit::GATE_DIR_DOWN => upc[1] |= 1 << 7,
                Bit::GATE_DIR_LEFT => upc[1] |= 1 << 6,
                Bit::METAL => upc[1] |= 1 << 5,
                Bit::METAL_DIR_UP => upc[1] |= 1 << 4,
                Bit::METAL_DIR_RIGHT => upc[1] |= 1 << 3,
                Bit::METAL_DIR_DOWN => upc[1] |= 1 << 2,
                Bit::METAL_DIR_LEFT => upc[1] |= 1 << 1,
                Bit::VIA => upc[1] |= 1 << 0,
                Bit::IO => upc[2] |= 1 << 7,
            }
        } else {
            match bit {
                Bit::SI_N => upc[0] &= !(1 << 7),
                Bit::SI_P => upc[0] &= !(1 << 6),
                Bit::SI_DIR_UP => upc[0] &= !(1 << 5),
                Bit::SI_DIR_RIGHT => upc[0] &= !(1 << 4),
                Bit::SI_DIR_DOWN => upc[0] &= !(1 << 3),
                Bit::SI_DIR_LEFT => upc[0] &= !(1 << 2),
                Bit::GATE_DIR_UP => upc[0] &= !(1 << 1),
                Bit::GATE_DIR_RIGHT => upc[0] &= !(1 << 0),
                Bit::GATE_DIR_DOWN => upc[1] &= !(1 << 7),
                Bit::GATE_DIR_LEFT => upc[1] &= !(1 << 6),
                Bit::METAL => upc[1] &= !(1 << 5),
                Bit::METAL_DIR_UP => upc[1] &= !(1 << 4),
                Bit::METAL_DIR_RIGHT => upc[1] &= !(1 << 3),
                Bit::METAL_DIR_DOWN => upc[1] &= !(1 << 2),
                Bit::METAL_DIR_LEFT => upc[1] &= !(1 << 1),
                Bit::VIA => upc[1] &= !(1 << 0),
                Bit::IO => upc[2] &= !(1 << 7),
            }
        }
    }
}

/// NormalizedCell exists purely as a programming convenience, especially for painting. When editing
/// cells it's easier to deal with the cell as a single struct, instead of as a collection of [0, 4]
/// Atoms. NormalizedCells should be treated as transient and not stored anywhere.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct NormalizedCell {
    pub metal: Metal,
    pub si: Silicon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metal {
    None,
    Trace { has_via: bool, placement: Placement },
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
        placement: Placement,
    },
    Mosfet {
        is_npn: bool,
        gate_placement: Placement,
        ec_placement: Placement,
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

impl From<UPC> for NormalizedCell {
    fn from(upc: UPC) -> Self {
        let mut cell = NormalizedCell::default();

        // Metal
        if upc.get_bit(Bit::METAL) {
            cell.metal = Metal::Trace {
                has_via: upc.get_bit(Bit::VIA),
                placement: Placement {
                    center: true,
                    up: upc.get_bit(Bit::METAL_DIR_UP),
                    right: upc.get_bit(Bit::METAL_DIR_RIGHT),
                    down: upc.get_bit(Bit::METAL_DIR_DOWN),
                    left: upc.get_bit(Bit::METAL_DIR_LEFT),
                },
            }
        }

        if upc.is_mosfet() {
            // MOSFET
            cell.si = Silicon::Mosfet {
                is_npn: upc.get_bit(Bit::SI_N),
                gate_placement: Placement {
                    center: true,
                    up: upc.get_bit(Bit::GATE_DIR_UP),
                    right: upc.get_bit(Bit::GATE_DIR_RIGHT),
                    down: upc.get_bit(Bit::GATE_DIR_DOWN),
                    left: upc.get_bit(Bit::GATE_DIR_LEFT),
                },
                ec_placement: Placement {
                    center: false,
                    up: upc.get_bit(Bit::SI_DIR_UP),
                    right: upc.get_bit(Bit::SI_DIR_RIGHT),
                    down: upc.get_bit(Bit::SI_DIR_DOWN),
                    left: upc.get_bit(Bit::SI_DIR_LEFT),
                },
            };
        } else if upc.get_bit(Bit::SI_N) || upc.get_bit(Bit::SI_P) {
            // Si trace only (non-mosfet)
            cell.si = Silicon::NP {
                is_n: upc.get_bit(Bit::SI_N),
                placement: Placement {
                    center: true,
                    up: upc.get_bit(Bit::SI_DIR_UP),
                    right: upc.get_bit(Bit::SI_DIR_RIGHT),
                    down: upc.get_bit(Bit::SI_DIR_DOWN),
                    left: upc.get_bit(Bit::SI_DIR_LEFT),
                },
            };
        }

        cell
    }
}

impl From<NormalizedCell> for UPC {
    fn from(cell: NormalizedCell) -> Self {
        let mut upc = Self::default();

        if let Metal::Trace { has_via, placement } = cell.metal {
            upc.set_bit(Bit::METAL);
            Bit::set(&mut upc, Bit::VIA, has_via);
            Bit::set(&mut upc, Bit::METAL_DIR_UP, placement.up);
            Bit::set(&mut upc, Bit::METAL_DIR_RIGHT, placement.right);
            Bit::set(&mut upc, Bit::METAL_DIR_DOWN, placement.down);
            Bit::set(&mut upc, Bit::METAL_DIR_LEFT, placement.left);
        }

        match cell.si {
            Silicon::NP { is_n, placement } => {
                if is_n {
                    upc.set_bit(Bit::SI_N);
                } else {
                    upc.set_bit(Bit::SI_P);
                }

                Bit::set(&mut upc, Bit::SI_DIR_UP, placement.up);
                Bit::set(&mut upc, Bit::SI_DIR_RIGHT, placement.right);
                Bit::set(&mut upc, Bit::SI_DIR_DOWN, placement.down);
                Bit::set(&mut upc, Bit::SI_DIR_LEFT, placement.left);
            }
            Silicon::Mosfet {
                is_npn,
                gate_placement,
                ec_placement,
            } => {
                if is_npn {
                    upc.set_bit(Bit::SI_N);
                } else {
                    upc.set_bit(Bit::SI_P);
                }

                Bit::set(&mut upc, Bit::GATE_DIR_UP, gate_placement.up);
                Bit::set(&mut upc, Bit::GATE_DIR_RIGHT, gate_placement.right);
                Bit::set(&mut upc, Bit::GATE_DIR_DOWN, gate_placement.down);
                Bit::set(&mut upc, Bit::GATE_DIR_LEFT, gate_placement.left);

                Bit::set(&mut upc, Bit::SI_DIR_UP, ec_placement.up);
                Bit::set(&mut upc, Bit::SI_DIR_RIGHT, ec_placement.right);
                Bit::set(&mut upc, Bit::SI_DIR_DOWN, ec_placement.down);
                Bit::set(&mut upc, Bit::SI_DIR_LEFT, ec_placement.left);
            }
            Silicon::None => {}
        }

        upc
    }
}

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
