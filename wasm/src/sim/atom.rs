use glam::IVec2;

/// The smallest divisible part of an overall network, a single conductor, IO pin, or Mosfet
/// connection. It also stores the `src_loc` for reference back to the original IC.
///
/// Atoms are unique within the Canvas from which they were generated. That is to say, all the
/// different `Path` instances derived from a single IO cell in a IC will have unique
/// AtomType::TerminalIoPin directions.
///
/// Atoms are geared toward simulation and don't map 1:1 with the IC drawing. For example, a
/// MOSFET cell in the IC will always correspond to exactly 4 Atoms in 3 different Paths:
/// - The Emitter Atom (E / C isn't discriminated) which connects to adjacent Silicon only.
/// - The Collector Atom (E / C isn't discriminated) which connects to adjacent Silicon only.
/// - The Base Atom which implicitly connects to the non-metal atom occupying the same cell
/// - A non-metal atom which implicitly connects to the Base Atom, and possibly other surrounding
///   Silicon as well.
/// In other words, 3 paths will always terminate at a MOSFET, and the one connected to the base
/// will always have at least 2 Atoms (NonMetal and TerminalMosfetBase).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Atom {
    pub src_loc: IVec2,
    pub atom_type: AtomType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AtomType {
    /// A terminal atom connection a conductor to an IO pin.
    TerminalIoPin,

    /// A terminal atom connecting a conductor to the BASE of a MOSFET. This is not used if the case
    /// where the base is draw such that it's also used as a conductor (the base spans "across" the
    /// MOSFET). In that case Si is used. Note that a base can only belong to exactly on Path, so we
    /// do not need to record `dir` to discriminate it from other connectors of the same terminal
    /// cell.
    TerminalMosfetBase { is_npn: bool },

    /// A terminal atom connecting a path to a, EMITTER or COLLECTOR of a MOSFET. This always
    /// connects to non-metal in the adjacent cell in the `dir` direction.
    TerminalMosfetEC {
        is_npn: bool,

        /// Emitters and collectors aren't discriminated like they are in real life, so the atom
        /// needs to store the offset vector for the non-metal is this E/C connects to.
        dir: IVec2,
    },

    /// A non-terminal non-metallic path through the cell. This can be either a single layer of
    /// silicon (if the cell isn't a MOSFET) or the silicon of the BASE for a MOSFET if that base
    /// spans "across" a MOSFET and is used on both sides.
    NonMetal,

    /// A non-terminal metallic path through the cell. This implicitly includes a via if present.
    Metal,
}
