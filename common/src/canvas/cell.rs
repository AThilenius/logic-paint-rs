use bevy::math::IVec2;
use smallvec::{smallvec, SmallVec};

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
            up: v == IVec2::Y,
            right: v == IVec2::X,
            down: v == -IVec2::Y,
            left: v == -IVec2::X,
        }
    }
}

impl CellDirs {
    #[inline(always)]
    pub fn get_vecs(&self) -> SmallVec<[IVec2; 4]> {
        let mut vec = smallvec![IVec2::default(); 4];

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
            (0, -1) => self.up = value,
            (1, 0) => self.right = value,
            (0, 1) => self.down = value,
            (-1, 0) => self.left = value,
            _ => panic!("Unsupported vector for set_direction"),
        }
    }

    // Returns true if this direction is 'in-line' with `dir`. This is used because gates (ie
    // silicon on top of silicon) cannot have bends in them. They can go left-right, or top-bottom.
    pub fn matches_gate_direction(&self, dir: IVec2) -> bool {
        match (dir.x, dir.y) {
            (0, -1) | (0, 1) => (self.up || self.down) && !(self.left || self.right),
            (-1, 0) | (1, 0) => (self.left || self.right) && !(self.up || self.down),
            _ => panic!("Unsupported vector for set_direction"),
        }
    }

    pub fn is_none(&self) -> bool {
        !(self.up || self.right || self.down || self.left)
    }
}

/// A single cell within a larger logic-paint canvas.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    /// True if silicon was n-doped (applies only to the lowest silicon layer).
    pub si_n: bool,

    /// True if silicon was p-doped (applies only to the lowest silicon layer).
    pub si_p: bool,

    /// True if the `si_type` layer is active.
    pub si_active: bool,

    /// The directions the (lowermost layer) of silicon are connected to adjacent cells.
    pub si_dirs: CellDirs,

    /// The directions (if any) the gate is connected. No connects implies no gate this cell. One
    /// connectin implies this cell is a transisitor (the gate is the opposite of the `si_type`).
    /// Two connections means the same thing, but only the opposite direction may accompany the
    /// initial connection. Three and four connections are invalid.
    pub gate_dirs: CellDirs,

    /// True if the gate (upper silicon layer) is active.
    pub gate_active: bool,

    /// True if this cell has a metal layer.
    pub metal: bool,

    /// True if the metal layer is active.
    pub metal_active: bool,

    /// The directions the metal layer is connected to adgacent cells.
    pub metal_dirs: CellDirs,

    /// True if this cell has a via going to the `si_type`. A via cannot connect to the gate layer.
    pub via: bool,
}

impl Cell {
    #[inline(always)]
    pub fn pack_into_4_bytes(&self, buf: &mut [u8]) {
        // Base layer silicon.
        // Note: last bit isn't used.
        buf[0] = 0
            | if self.si_n { 1 << 7 } else { 0 }
            | if self.si_p { 1 << 6 } else { 0 }
            | if self.si_active { 1 << 5 } else { 0 }
            | if self.si_dirs.up { 1 << 4 } else { 0 }
            | if self.si_dirs.right { 1 << 3 } else { 0 }
            | if self.si_dirs.down { 1 << 2 } else { 0 }
            | if self.si_dirs.left { 1 << 1 } else { 0 };

        // Upper layer silicon.
        // Note: last 3 bits aren't used.
        buf[1] = 0
            | if self.gate_dirs.up { 1 << 7 } else { 0 }
            | if self.gate_dirs.right { 1 << 6 } else { 0 }
            | if self.gate_dirs.down { 1 << 5 } else { 0 }
            | if self.gate_dirs.left { 1 << 4 } else { 0 }
            | if self.gate_active { 1 << 3 } else { 0 };

        // Metal and vias.
        // Note: last bit isn't used.
        buf[2] = 0
            | if self.metal { 1 << 7 } else { 0 }
            | if self.metal_dirs.up { 1 << 6 } else { 0 }
            | if self.metal_dirs.right { 1 << 5 } else { 0 }
            | if self.metal_dirs.down { 1 << 4 } else { 0 }
            | if self.metal_dirs.left { 1 << 3 } else { 0 }
            | if self.metal_active { 1 << 2 } else { 0 }
            | if self.via { 1 << 1 } else { 0 };

        // Alpha byte is unused and set to 255;
        buf[3] = 255_u8;
    }
}
