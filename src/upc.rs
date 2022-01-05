pub const UPC_BYTE_LEN: usize = 4;
pub const LOG_UPC_BYTE_LEN: usize = 2;

/// Universal Packed Cell format stores each cell as a bit packed [u8; 4], ready for direct blitting
/// to a GPU RGBu8 texture. Stored as [u8; 4] instead of u32 for endian agnosticism during blitting.
/// Does not encode ActiveMask data. The first 16 bits are also encoded as part of Blueprint
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
        let mut upc = Default::default();

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
