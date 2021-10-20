use glam::IVec2;

use crate::substrate::{Atom, MosfetPart};

use super::{placement::Placement, Cell};

/// NormalizedCell exists purely as a programming convenience, especially for painting. When editing
/// cells it's easier to deal with the cell as a single struct, instead of as a collection of [0, 4]
/// Atoms. NormalizedCells should be treated as transient and not stored anywhere.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct NormalizedCell {
    pub cell_loc: IVec2,
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

impl From<Cell> for NormalizedCell {
    fn from(cell: Cell) -> Self {
        let mut nc = Self::default();

        for atom in cell {
            nc.cell_loc = atom.cell_loc;

            if atom.metal != Placement::NONE {
                nc.metal = Metal::Trace {
                    has_via: atom.metal != Placement::NONE && atom.si != Placement::NONE,
                    placement: atom.metal,
                };
            }

            if atom.si != Placement::NONE {
                if atom.mosfet_part == MosfetPart::None {
                    nc.si = Silicon::NP {
                        is_n: atom.is_si_n,
                        placement: atom.si,
                    };
                } else if atom.mosfet_part == MosfetPart::Base {
                    nc.si = Silicon::Mosfet {
                        is_npn: !atom.is_si_n,
                        gate_placement: atom.si,
                        ec_placement: if atom.si.left || atom.si.right {
                            Placement::UP | Placement::DOWN
                        } else {
                            Placement::LEFT | Placement::RIGHT
                        },
                    };
                }

                // The Left/Right EC atoms are ignored, we only need the base.
            }
        }

        nc
    }
}

impl From<NormalizedCell> for Cell {
    fn from(nc: NormalizedCell) -> Self {
        let mut cell = Self::default();

        match (nc.metal, nc.si) {
            (Metal::None, Silicon::None) => {}
            (Metal::None, Silicon::NP { is_n, placement }) => cell.push(Atom {
                cell_loc: nc.cell_loc,
                si: placement,
                is_si_n: is_n,
                ..Default::default()
            }),
            (
                Metal::None,
                Silicon::Mosfet {
                    is_npn,
                    gate_placement,
                    ec_placement,
                },
            ) => {
                // Base
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: gate_placement,
                    is_si_n: !is_npn,
                    mosfet_part: MosfetPart::Base,
                    ..Default::default()
                });

                // Left EC (left or up)
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: if ec_placement.up {
                        Placement::UP
                    } else {
                        Placement::LEFT
                    },
                    is_si_n: is_npn,
                    mosfet_part: MosfetPart::LeftEC,
                    ..Default::default()
                });

                // Right EC (right or down)
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: if ec_placement.down {
                        Placement::DOWN
                    } else {
                        Placement::RIGHT
                    },
                    is_si_n: is_npn,
                    mosfet_part: MosfetPart::RightEC,
                    ..Default::default()
                });
            }
            (Metal::Trace { placement, .. }, Silicon::None) => cell.push(Atom {
                cell_loc: nc.cell_loc,
                metal: placement,
                ..Default::default()
            }),
            (
                Metal::Trace {
                    has_via,
                    placement: metal_placement,
                },
                Silicon::NP {
                    is_n,
                    placement: si_placement,
                },
            ) => {
                if has_via {
                    cell.push(Atom {
                        cell_loc: nc.cell_loc,
                        metal: metal_placement,
                        si: si_placement,
                        is_si_n: is_n,
                        ..Default::default()
                    });
                } else {
                    cell.push(Atom {
                        cell_loc: nc.cell_loc,
                        metal: metal_placement,
                        ..Default::default()
                    });
                    cell.push(Atom {
                        cell_loc: nc.cell_loc,
                        si: si_placement,
                        is_si_n: is_n,
                        ..Default::default()
                    });
                }
            }
            (
                Metal::Trace {
                    placement: metal_placement,
                    ..
                },
                Silicon::Mosfet {
                    is_npn,
                    gate_placement,
                    ec_placement,
                },
            ) => {
                // Vias cannot connect to MOSFETS, so we don't need to check them.
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    metal: metal_placement,
                    ..Default::default()
                });

                // Base
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: gate_placement,
                    is_si_n: !is_npn,
                    mosfet_part: MosfetPart::Base,
                    ..Default::default()
                });

                // Left EC (left or up)
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: if ec_placement.up {
                        Placement::UP
                    } else {
                        Placement::LEFT
                    },
                    is_si_n: is_npn,
                    mosfet_part: MosfetPart::LeftEC,
                    ..Default::default()
                });

                // Right EC (right or down)
                cell.push(Atom {
                    cell_loc: nc.cell_loc,
                    si: if ec_placement.down {
                        Placement::DOWN
                    } else {
                        Placement::RIGHT
                    },
                    is_si_n: is_npn,
                    mosfet_part: MosfetPart::RightEC,
                    ..Default::default()
                });
            }
        }

        cell
    }
}

#[cfg(test)]
mod tests {
    use glam::IVec2;

    use crate::substrate::{placement::Placement, Cell, Metal, Silicon};

    use super::NormalizedCell;

    #[test]
    fn default_empty() {
        let nc = NormalizedCell::default();
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 0);
        assert_eq!(NormalizedCell::from(cell), NormalizedCell::default());
    }

    #[test]
    fn cell_metal() {
        // Metal
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::Trace {
                has_via: false,
                placement: Placement::UP,
            },
            si: Silicon::None,
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 1);
        assert_eq!(NormalizedCell::from(cell), nc);
    }

    #[test]
    fn cell_si() {
        // Si
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::None,
            si: Silicon::NP {
                is_n: true,
                placement: Placement::RIGHT,
            },
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 1);
        assert_eq!(NormalizedCell::from(cell), nc);
    }

    #[test]
    fn cell_no_via() {
        // No Via
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::Trace {
                has_via: false,
                placement: Placement::UP,
            },
            si: Silicon::NP {
                is_n: true,
                placement: Placement::RIGHT,
            },
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 2);
        assert_eq!(NormalizedCell::from(cell), nc);
    }

    #[test]
    fn cell_with_via() {
        // Via
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::Trace {
                has_via: true,
                placement: Placement::UP,
            },
            si: Silicon::NP {
                is_n: true,
                placement: Placement::RIGHT,
            },
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 1);
        assert_eq!(NormalizedCell::from(cell), nc);
    }

    #[test]
    fn cell_with_mosfet() {
        // Mosfet
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::None,
            si: Silicon::Mosfet {
                is_npn: true,
                gate_placement: Placement::UP | Placement::DOWN,
                ec_placement: Placement::LEFT | Placement::RIGHT,
            },
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 3);
        assert_eq!(NormalizedCell::from(cell), nc);
    }

    #[test]
    fn cell_with_mosfet_and_metal() {
        // Mosfet
        let nc = NormalizedCell {
            cell_loc: IVec2::ONE,
            metal: Metal::Trace {
                has_via: false,
                placement: Placement::UP,
            },
            si: Silicon::Mosfet {
                is_npn: true,
                gate_placement: Placement::UP | Placement::DOWN,
                ec_placement: Placement::LEFT | Placement::RIGHT,
            },
        };
        let cell = Cell::from(nc.clone());
        assert_eq!(cell.len(), 4);
        assert_eq!(NormalizedCell::from(cell), nc);
    }
}
