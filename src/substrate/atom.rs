use arrayvec::ArrayVec;
use glam::IVec2;

use super::placement::Placement;

/// A cell can have at most 4 atoms, if it's a MOSFET (3 atoms for the left EC, right EC and base)
/// with a metal layer above it.
const MAX_ATOMS_PER_CELL: usize = 4;

/// An Atom belongs to exactly one conductive path. It may be a middle-link in the path, or a leaf
/// node of the path connecting to a MOSFET, or both.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Atom {
    /// The cell location this atom resides in.
    pub cell_loc: IVec2,

    /// The metal layer placements of this atom connects. Non-zero metal and non-zero Si in on
    /// atom implies the presence of a Via.
    pub metal: Placement,

    /// The si layer placements of this atom connects. Non-zero metal and non-zero Si in on atom
    /// implies the presence of a Via.
    pub si: Placement,

    /// Set to true if the si above are N-Type. Meaningless when si is zero.
    pub is_si_n: bool,

    /// The part of a MOSFET that this atom (and thus the Path it belongs to) joins with, if any.
    pub mosfet_part: MosfetPart,
}

/// Convenience type when accessing atoms organized into cells.
pub type Cell = ArrayVec<Atom, MAX_ATOMS_PER_CELL>;

/// The different parts (as far as a conductive path is concerned) of a MOSFET.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum MosfetPart {
    None = 0,
    Base = 1,
    LeftEC = 2,
    RightEC = 3,
}

impl Default for MosfetPart {
    fn default() -> Self {
        Self::None
    }
}

impl Atom {
    /// Returns true if the 'other' Atom is a neighbor of self.
    pub fn neighbor_of(&self, other: &Atom) -> bool {
        let dir = other.cell_loc - self.cell_loc;

        (self.metal.has_cardinal(dir) && other.metal.has_cardinal(-dir))
            || (self.si.has_cardinal(dir) && other.si.has_cardinal(-dir))
    }
}
