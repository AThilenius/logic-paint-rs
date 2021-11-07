/// UPC format: Universal Packed Cell format stores each cell as a bit packed u32, ready for direct
/// blitting to a GPU 32 bit integer type texture. Does not encode ActiveMask data. Modules are
/// rendered separately from cells, allowing each module to render itself differently.
pub type UPC = u32;

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum Bit {
    SI_N = 1 << 0,
    SI_P = 1 << 1,
    SI_DIR_UP = 1 << 2,
    SI_DIR_RIGHT = 1 << 3,
    SI_DIR_DOWN = 1 << 4,
    SI_DIR_LEFT = 1 << 5,
    GATE_DIR_UP = 1 << 6,
    GATE_DIR_RIGHT = 1 << 7,
    GATE_DIR_DOWN = 1 << 8,
    GATE_DIR_LEFT = 1 << 9,
    METAL = 1 << 10,
    METAL_DIR_UP = 1 << 11,
    METAL_DIR_RIGHT = 1 << 12,
    METAL_DIR_DOWN = 1 << 13,
    METAL_DIR_LEFT = 1 << 14,
    VIA = 1 << 15,
    IO = 1 << 16,
}

impl Bit {
    #[inline(always)]
    pub fn get(upc: UPC, bit: Bit) -> bool {
        (upc & bit as u32) > 0
    }

    #[inline(always)]
    pub fn set(upc: &mut UPC, bit: Bit, value: bool) {
        if value {
            *upc |= bit as u32;
        } else {
            *upc &= !(bit as u32);
        }
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct UnpackedCell {
    pub si_n: bool,
    pub si_p: bool,
    pub si_dir_up: bool,
    pub si_dir_right: bool,
    pub si_dir_down: bool,
    pub si_dir_left: bool,
    pub gate_dir_up: bool,
    pub gate_dir_right: bool,
    pub gate_dir_down: bool,
    pub gate_dir_left: bool,
    pub metal: bool,
    pub metal_dir_up: bool,
    pub metal_dir_right: bool,
    pub metal_dir_down: bool,
    pub metal_dir_left: bool,
    pub via: bool,
    pub io: bool,
}

impl From<UPC> for UnpackedCell {
    fn from(upc: UPC) -> Self {
        Self {
            si_n: Bit::get(upc, Bit::SI_N),
            si_p: Bit::get(upc, Bit::SI_P),
            si_dir_up: Bit::get(upc, Bit::SI_DIR_UP),
            si_dir_right: Bit::get(upc, Bit::SI_DIR_RIGHT),
            si_dir_down: Bit::get(upc, Bit::SI_DIR_DOWN),
            si_dir_left: Bit::get(upc, Bit::SI_DIR_LEFT),
            gate_dir_up: Bit::get(upc, Bit::GATE_DIR_UP),
            gate_dir_right: Bit::get(upc, Bit::GATE_DIR_RIGHT),
            gate_dir_down: Bit::get(upc, Bit::GATE_DIR_DOWN),
            gate_dir_left: Bit::get(upc, Bit::GATE_DIR_LEFT),
            metal: Bit::get(upc, Bit::METAL),
            metal_dir_up: Bit::get(upc, Bit::METAL_DIR_UP),
            metal_dir_right: Bit::get(upc, Bit::METAL_DIR_RIGHT),
            metal_dir_down: Bit::get(upc, Bit::METAL_DIR_DOWN),
            metal_dir_left: Bit::get(upc, Bit::METAL_DIR_LEFT),
            via: Bit::get(upc, Bit::VIA),
            io: Bit::get(upc, Bit::IO),
        }
    }
}

impl Into<UPC> for UnpackedCell {
    fn into(self) -> UPC {
        let mut upc = 0u32;

        Bit::set(&mut upc, Bit::SI_N, self.si_n);
        Bit::set(&mut upc, Bit::SI_P, self.si_p);
        Bit::set(&mut upc, Bit::SI_DIR_UP, self.si_dir_up);
        Bit::set(&mut upc, Bit::SI_DIR_RIGHT, self.si_dir_right);
        Bit::set(&mut upc, Bit::SI_DIR_DOWN, self.si_dir_down);
        Bit::set(&mut upc, Bit::SI_DIR_LEFT, self.si_dir_left);
        Bit::set(&mut upc, Bit::GATE_DIR_UP, self.gate_dir_up);
        Bit::set(&mut upc, Bit::GATE_DIR_RIGHT, self.gate_dir_right);
        Bit::set(&mut upc, Bit::GATE_DIR_DOWN, self.gate_dir_down);
        Bit::set(&mut upc, Bit::GATE_DIR_LEFT, self.gate_dir_left);
        Bit::set(&mut upc, Bit::METAL, self.metal);
        Bit::set(&mut upc, Bit::METAL_DIR_UP, self.metal_dir_up);
        Bit::set(&mut upc, Bit::METAL_DIR_RIGHT, self.metal_dir_right);
        Bit::set(&mut upc, Bit::METAL_DIR_DOWN, self.metal_dir_down);
        Bit::set(&mut upc, Bit::METAL_DIR_LEFT, self.metal_dir_left);
        Bit::set(&mut upc, Bit::VIA, self.via);
        Bit::set(&mut upc, Bit::IO, self.io);

        upc
    }
}
