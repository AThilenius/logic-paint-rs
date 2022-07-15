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
        self.get_bit(Bit::MOSFET_HORIZONTAL) || self.get_bit(Bit::MOSFET_VERTICAL)
    }

    pub fn rotate(&self) -> UPC {
        let mut upc = self.clone();
        Bit::set(
            &mut upc,
            Bit::MOSFET_HORIZONTAL,
            self.get_bit(Bit::MOSFET_VERTICAL),
        );
        Bit::set(
            &mut upc,
            Bit::MOSFET_VERTICAL,
            self.get_bit(Bit::MOSFET_HORIZONTAL),
        );

        Bit::set(&mut upc, Bit::SI_DIR_UP, self.get_bit(Bit::SI_DIR_LEFT));
        Bit::set(&mut upc, Bit::SI_DIR_RIGHT, self.get_bit(Bit::SI_DIR_UP));
        Bit::set(&mut upc, Bit::SI_DIR_DOWN, self.get_bit(Bit::SI_DIR_RIGHT));
        Bit::set(&mut upc, Bit::SI_DIR_LEFT, self.get_bit(Bit::SI_DIR_DOWN));

        Bit::set(
            &mut upc,
            Bit::METAL_DIR_UP,
            self.get_bit(Bit::METAL_DIR_LEFT),
        );
        Bit::set(
            &mut upc,
            Bit::METAL_DIR_RIGHT,
            self.get_bit(Bit::METAL_DIR_UP),
        );
        Bit::set(
            &mut upc,
            Bit::METAL_DIR_DOWN,
            self.get_bit(Bit::METAL_DIR_RIGHT),
        );
        Bit::set(
            &mut upc,
            Bit::METAL_DIR_LEFT,
            self.get_bit(Bit::METAL_DIR_DOWN),
        );

        upc
    }

    pub fn mirror(&self) -> UPC {
        let mut upc = self.clone();

        Bit::set(&mut upc, Bit::SI_DIR_UP, self.get_bit(Bit::SI_DIR_DOWN));
        Bit::set(&mut upc, Bit::SI_DIR_DOWN, self.get_bit(Bit::SI_DIR_UP));

        Bit::set(
            &mut upc,
            Bit::METAL_DIR_UP,
            self.get_bit(Bit::METAL_DIR_DOWN),
        );
        Bit::set(
            &mut upc,
            Bit::METAL_DIR_DOWN,
            self.get_bit(Bit::METAL_DIR_UP),
        );

        upc
    }
}

#[allow(non_camel_case_types)]
pub enum Bit {
    SI_N,
    SI_P,
    MOSFET_HORIZONTAL,
    MOSFET_VERTICAL,
    SI_DIR_UP,
    SI_DIR_RIGHT,
    SI_DIR_DOWN,
    SI_DIR_LEFT,
    METAL,
    METAL_DIR_UP,
    METAL_DIR_RIGHT,
    METAL_DIR_DOWN,
    METAL_DIR_LEFT,
    VIA,
    IO,
    MODULE_ROOT,
}

impl Bit {
    #[inline(always)]
    pub fn get(upc: UPC, bit: Bit) -> bool {
        let upc = upc.0;
        match bit {
            Bit::SI_N => upc[0] & (1 << 7) > 0,
            Bit::SI_P => upc[0] & (1 << 6) > 0,
            Bit::MOSFET_HORIZONTAL => upc[0] & (1 << 5) > 0,
            Bit::MOSFET_VERTICAL => upc[0] & (1 << 4) > 0,
            Bit::SI_DIR_UP => upc[0] & (1 << 3) > 0,
            Bit::SI_DIR_RIGHT => upc[0] & (1 << 2) > 0,
            Bit::SI_DIR_DOWN => upc[0] & (1 << 1) > 0,
            Bit::SI_DIR_LEFT => upc[0] & (1 << 0) > 0,

            Bit::METAL => upc[1] & (1 << 7) > 0,
            Bit::METAL_DIR_UP => upc[1] & (1 << 6) > 0,
            Bit::METAL_DIR_RIGHT => upc[1] & (1 << 5) > 0,
            Bit::METAL_DIR_DOWN => upc[1] & (1 << 4) > 0,
            Bit::METAL_DIR_LEFT => upc[1] & (1 << 3) > 0,
            Bit::VIA => upc[1] & (1 << 2) > 0,

            Bit::IO => upc[2] & (1 << 7) > 0,
            Bit::MODULE_ROOT => upc[2] & (1 << 6) > 0,
        }
    }

    #[inline(always)]
    pub fn set(upc: &mut UPC, bit: Bit, value: bool) {
        let upc = &mut upc.0;
        if value {
            match bit {
                Bit::SI_N => upc[0] |= 1 << 7,
                Bit::SI_P => upc[0] |= 1 << 6,
                Bit::MOSFET_HORIZONTAL => upc[0] |= 1 << 5,
                Bit::MOSFET_VERTICAL => upc[0] |= 1 << 4,
                Bit::SI_DIR_UP => upc[0] |= 1 << 3,
                Bit::SI_DIR_RIGHT => upc[0] |= 1 << 2,
                Bit::SI_DIR_DOWN => upc[0] |= 1 << 1,
                Bit::SI_DIR_LEFT => upc[0] |= 1 << 0,

                Bit::METAL => upc[1] |= 1 << 7,
                Bit::METAL_DIR_UP => upc[1] |= 1 << 6,
                Bit::METAL_DIR_RIGHT => upc[1] |= 1 << 5,
                Bit::METAL_DIR_DOWN => upc[1] |= 1 << 4,
                Bit::METAL_DIR_LEFT => upc[1] |= 1 << 3,
                Bit::VIA => upc[1] |= 1 << 2,

                Bit::IO => upc[2] |= 1 << 7,
                Bit::MODULE_ROOT => upc[2] |= 1 << 6,
            }
        } else {
            match bit {
                Bit::SI_N => upc[0] &= !(1 << 7),
                Bit::SI_P => upc[0] &= !(1 << 6),
                Bit::MOSFET_HORIZONTAL => upc[0] &= !(1 << 5),
                Bit::MOSFET_VERTICAL => upc[0] &= !(1 << 4),
                Bit::SI_DIR_UP => upc[0] &= !(1 << 3),
                Bit::SI_DIR_RIGHT => upc[0] &= !(1 << 2),
                Bit::SI_DIR_DOWN => upc[0] &= !(1 << 1),
                Bit::SI_DIR_LEFT => upc[0] &= !(1 << 0),

                Bit::METAL => upc[1] &= !(1 << 7),
                Bit::METAL_DIR_UP => upc[1] &= !(1 << 6),
                Bit::METAL_DIR_RIGHT => upc[1] &= !(1 << 5),
                Bit::METAL_DIR_DOWN => upc[1] &= !(1 << 4),
                Bit::METAL_DIR_LEFT => upc[1] &= !(1 << 3),
                Bit::VIA => upc[1] &= !(1 << 2),

                Bit::IO => upc[2] &= !(1 << 7),
                Bit::MODULE_ROOT => upc[2] &= !(1 << 6),
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
    pub module_root: bool,
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
        is_horizontal: bool,
        gate_placement: Placement,
        ec_placement: Placement,
    },
}

impl Default for Silicon {
    fn default() -> Self {
        Self::None
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
                    up: upc.get_bit(Bit::METAL_DIR_UP),
                    right: upc.get_bit(Bit::METAL_DIR_RIGHT),
                    down: upc.get_bit(Bit::METAL_DIR_DOWN),
                    left: upc.get_bit(Bit::METAL_DIR_LEFT),
                },
            }
        }

        if upc.is_mosfet() {
            // MOSFET
            let is_horizontal = upc.get_bit(Bit::MOSFET_HORIZONTAL);
            cell.si = Silicon::Mosfet {
                is_npn: upc.get_bit(Bit::SI_N),
                is_horizontal,
                gate_placement: Placement {
                    up: !is_horizontal && upc.get_bit(Bit::SI_DIR_UP),
                    right: is_horizontal && upc.get_bit(Bit::SI_DIR_RIGHT),
                    down: !is_horizontal && upc.get_bit(Bit::SI_DIR_DOWN),
                    left: is_horizontal && upc.get_bit(Bit::SI_DIR_LEFT),
                },
                ec_placement: Placement {
                    up: is_horizontal && upc.get_bit(Bit::SI_DIR_UP),
                    right: !is_horizontal && upc.get_bit(Bit::SI_DIR_RIGHT),
                    down: is_horizontal && upc.get_bit(Bit::SI_DIR_DOWN),
                    left: !is_horizontal && upc.get_bit(Bit::SI_DIR_LEFT),
                },
            };
        } else if upc.get_bit(Bit::SI_N) || upc.get_bit(Bit::SI_P) {
            // Si trace only (non-mosfet)
            cell.si = Silicon::NP {
                is_n: upc.get_bit(Bit::SI_N),
                placement: Placement {
                    up: upc.get_bit(Bit::SI_DIR_UP),
                    right: upc.get_bit(Bit::SI_DIR_RIGHT),
                    down: upc.get_bit(Bit::SI_DIR_DOWN),
                    left: upc.get_bit(Bit::SI_DIR_LEFT),
                },
            };
        }

        cell.module_root = upc.get_bit(Bit::MODULE_ROOT);

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
                is_horizontal,
                gate_placement,
                ec_placement,
            } => {
                if is_npn {
                    upc.set_bit(Bit::SI_N);
                } else {
                    upc.set_bit(Bit::SI_P);
                }

                Bit::set(&mut upc, Bit::MOSFET_HORIZONTAL, is_horizontal);
                Bit::set(&mut upc, Bit::MOSFET_VERTICAL, !is_horizontal);

                Bit::set(
                    &mut upc,
                    Bit::SI_DIR_UP,
                    ec_placement.up || gate_placement.up,
                );
                Bit::set(
                    &mut upc,
                    Bit::SI_DIR_RIGHT,
                    ec_placement.right || gate_placement.right,
                );
                Bit::set(
                    &mut upc,
                    Bit::SI_DIR_DOWN,
                    ec_placement.down || gate_placement.down,
                );
                Bit::set(
                    &mut upc,
                    Bit::SI_DIR_LEFT,
                    ec_placement.left || gate_placement.left,
                );
            }
            Silicon::None => {}
        }

        Bit::set(&mut upc, Bit::MODULE_ROOT, cell.module_root);

        upc
    }
}

/// Represents the various placements of Metal and Si within a Cell, including the 4 cardinal
/// directions, and the center "self" location (which is implicit when any cardinal direction is
/// set, but can also stand alone).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Placement {
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

impl Placement {
    pub const NONE: Self = Self {
        up: false,
        right: false,
        down: false,
        left: false,
    };

    pub const CENTER: Self = Self {
        up: false,
        right: false,
        down: false,
        left: false,
    };

    pub fn from_cardinal(dir: IVec2) -> Self {
        let mut placement = Placement::NONE;
        placement.set_cardinal(dir);
        placement
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
    }

    pub fn clear_cardinal(&mut self, dir: IVec2) {
        match (dir.x, dir.y) {
            (0, 1) => self.up = false,
            (1, 0) => self.right = false,
            (0, -1) => self.down = false,
            (-1, 0) => self.left = false,
            _ => panic!("Non-unit cardinal direction vector"),
        }
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
            left: self.left | rhs.left,
            up: self.up | rhs.up,
            right: self.right | rhs.right,
            down: self.down | rhs.down,
        }
    }
}
