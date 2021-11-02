/// Bespoke bit-packing serializer for Cells, which can be compacted down into 4 bytes using a few
/// tricks, saving an immense amount of space. Everything else should be serialized with Protobuf.
use std::collections::HashMap;
use std::iter::FromIterator;

use arrayvec::ArrayVec;
use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    substrate::{MosfetPart, Placement},
    utils::ChunkedHashMap,
    warn,
    wgl2::LOG_CHUNK_SIZE,
};

use super::{Atom, Cell, IntegratedCircuit, PinModule};

const FLAG_METAL: u32 = 1 << 0;
const FLAG_METAL_UP: u32 = 1 << 1;
const FLAG_METAL_RIGHT: u32 = 1 << 2;
const FLAG_METAL_DOWN: u32 = 1 << 3;
const FLAG_METAL_LEFT: u32 = 1 << 4;
const FLAG_VIA: u32 = 1 << 5;
const FLAG_SI_NP: u32 = 1 << 6;
const FLAG_SI_MOSFET: u32 = 1 << 7;
const FLAG_SI_N: u32 = 1 << 8;

// Used with FLAG_SI_NP
const FLAG_SI_UP: u32 = 1 << 9;
const FLAG_SI_RIGHT: u32 = 1 << 10;
const FLAG_SI_DOWN: u32 = 1 << 11;
const FLAG_SI_LEFT: u32 = 1 << 12;

// Use with FLAG_SI_MOSFET
const FLAG_SI_MOSFET_HORIZONTAL: u32 = 1 << 9;
const FLAG_SI_MOSFET_BASE_UL: u32 = 1 << 10;
const FLAG_SI_MOSFET_BASE_DR: u32 = 1 << 11;

#[derive(Serialize, Deserialize)]
struct IntegratedCircuitSurrogate {
    pub chunks: Vec<CellChunkSurrogate>,
    pub pin_modules: Vec<PinModule>,
}

#[derive(Serialize, Deserialize)]
struct CellChunkSurrogate {
    pub chunk_loc: IVec2,
    pub bit_packed_cells: Vec<u32>,
}

pub fn serialize_ic(ic: &IntegratedCircuit) -> Vec<u8> {
    let ic_surrogate = IntegratedCircuitSurrogate {
        chunks: ic
            .chunk_locs()
            .cloned()
            .map(|chunk_loc| {
                let loc_anchor =
                    IVec2::new(chunk_loc.x << LOG_CHUNK_SIZE, chunk_loc.y << LOG_CHUNK_SIZE);
                let bit_packed_cells = ic
                    .get_cell_chunk_by_chunk_location(&chunk_loc)
                    .unwrap()
                    .values()
                    .map(|cell| serialize_cell(&cell, &loc_anchor))
                    .collect();

                CellChunkSurrogate {
                    chunk_loc,
                    bit_packed_cells,
                }
            })
            .collect(),
        pin_modules: ic.pin_modules().cloned().collect(),
    };

    match bincode::serialize(&ic_surrogate) {
        Err(e) => {
            warn!("Failed to serialize IC: {:#?}", e);
            return vec![];
        }
        Ok(data) => data,
    }
}

pub fn deserialize_ic(data: &Vec<u8>) -> IntegratedCircuit {
    let ic_surrogate: IntegratedCircuitSurrogate = {
        let ics = bincode::deserialize(data);
        if let Err(e) = ics {
            warn!("Failed to deserialize IC: {:#?}", e);
            return Default::default();
        }

        ics.unwrap()
    };

    // Manually create a ChunkedHashMap because it's *much* faster.
    let mut chunks = Vec::new();
    let mut chunk_lookup_by_chunk_idx = HashMap::new();

    for chunk in ic_surrogate.chunks {
        let loc_anchor = IVec2::new(
            chunk.chunk_loc.x << LOG_CHUNK_SIZE,
            chunk.chunk_loc.y << LOG_CHUNK_SIZE,
        );

        // Deserialize cells
        chunk_lookup_by_chunk_idx.insert(chunk.chunk_loc.clone(), chunks.len());
        chunks.push(HashMap::from_iter(chunk.bit_packed_cells.into_iter().map(
            |bits| {
                let cell = deserialize_cell(bits, &loc_anchor);
                (cell[0].cell_loc, cell)
            },
        )));
    }

    IntegratedCircuit::new(
        ChunkedHashMap {
            chunks,
            chunk_lookup_by_chunk_idx,
        },
        ic_surrogate.pin_modules,
    )
}

/// Serialize the NormalizedCell into 4 bytes:
/// [0-7], [8-15]: X, Y coords as the difference between loc_anchor and self.cell_loc.
/// [16-31]: Bit packed layout data.
fn serialize_cell(cell: &Cell, offset: &IVec2) -> u32 {
    // Sanity check
    if cell.len() == 0 {
        warn!("Cannot serialize 0-size cell!");
        return 0;
    }

    let mut bits = 0u32;
    let loc = cell[0].cell_loc - *offset;

    bits |= (loc.x as u8) as u32;

    bits = bits << 8;
    bits |= (loc.y as u8) as u32;

    bits = bits << 16;
    for atom in cell {
        if atom.metal != Placement::NONE {
            bits |= FLAG_METAL;
            if atom.metal.up {
                bits |= FLAG_METAL_UP;
            };
            if atom.metal.right {
                bits |= FLAG_METAL_RIGHT;
            };
            if atom.metal.down {
                bits |= FLAG_METAL_DOWN;
            };
            if atom.metal.left {
                bits |= FLAG_METAL_LEFT;
            };
            if atom.si != Placement::NONE {
                bits |= FLAG_VIA;
            }
        }

        if atom.si != Placement::NONE {
            if atom.mosfet_part == MosfetPart::None {
                bits |= FLAG_SI_NP;
                if atom.is_si_n {
                    bits |= FLAG_SI_N;
                }
                if atom.si.up {
                    bits |= FLAG_SI_UP;
                }
                if atom.si.right {
                    bits |= FLAG_SI_RIGHT;
                }
                if atom.si.down {
                    bits |= FLAG_SI_DOWN;
                }
                if atom.si.left {
                    bits |= FLAG_SI_LEFT;
                }
            } else if atom.mosfet_part == MosfetPart::Base {
                // atom.si in this case is the base.
                bits |= FLAG_SI_MOSFET;
                if atom.is_si_n {
                    bits |= FLAG_SI_N;
                }
                if atom.si.up || atom.si.down {
                    bits |= FLAG_SI_MOSFET_HORIZONTAL;
                } else {
                    debug_assert!(atom.si.left || atom.si.right, "Invalid MOSFET base");
                }
                if atom.si.up || atom.si.left {
                    bits |= FLAG_SI_MOSFET_BASE_UL;
                }
                if atom.si.down || atom.si.right {
                    bits |= FLAG_SI_MOSFET_BASE_DR;
                }
            }

            // The Left/Right EC atoms are ignored, we only need the base.
        }
    }

    bits
}

fn deserialize_cell(bits: u32, loc_anchor: &IVec2) -> Cell {
    let x = (bits >> 24) & 255;
    let y = (bits >> 16) & 255;
    let cell_loc = IVec2::new(x as i32 + loc_anchor.x, y as i32 + loc_anchor.y);

    let mut atoms: Cell = ArrayVec::new();

    let metal_placement = Placement {
        center: bits & FLAG_METAL > 0,
        up: bits & FLAG_METAL_UP > 0,
        right: bits & FLAG_METAL_RIGHT > 0,
        down: bits & FLAG_METAL_DOWN > 0,
        left: bits & FLAG_METAL_LEFT > 0,
    };
    let si_np_placement = Placement {
        center: bits & FLAG_SI_NP > 0,
        up: bits & FLAG_SI_UP > 0,
        right: bits & FLAG_SI_RIGHT > 0,
        down: bits & FLAG_SI_DOWN > 0,
        left: bits & FLAG_SI_LEFT > 0,
    };

    if bits & FLAG_METAL > 0 {
        // Metal atom of some type.
        if bits & FLAG_VIA > 0 {
            // Metal + Si atom.
            atoms.push(Atom {
                cell_loc,
                metal: metal_placement,
                si: si_np_placement,
                is_si_n: bits & FLAG_SI_N > 0,
                mosfet_part: MosfetPart::None,
            });
        } else {
            // Metal only atom.
            atoms.push(Atom {
                cell_loc,
                metal: metal_placement,
                ..Default::default()
            });
        }
    } else {
        // Non-metal atom of some type.
        if bits & FLAG_SI_NP > 0 {
            // Si trace
            atoms.push(Atom {
                cell_loc,
                si: si_np_placement,
                is_si_n: bits & FLAG_SI_N > 0,
                ..Default::default()
            });
        } else if bits & FLAG_SI_MOSFET > 0 {
            // Si MOSFET base (add all 3 atoms for Base/E/C)
            let is_base_n = bits & FLAG_SI_N > 0;
            let is_horizontal = bits & FLAG_SI_MOSFET_HORIZONTAL > 0;

            // Left or up EC
            atoms.push(Atom {
                cell_loc,
                si: if is_horizontal {
                    Placement::LEFT
                } else {
                    Placement::UP
                },
                is_si_n: !is_base_n,
                mosfet_part: MosfetPart::LeftEC,
                ..Default::default()
            });

            // Right or down EC
            atoms.push(Atom {
                cell_loc,
                si: if is_horizontal {
                    Placement::RIGHT
                } else {
                    Placement::DOWN
                },
                is_si_n: !is_base_n,
                mosfet_part: MosfetPart::RightEC,
                ..Default::default()
            });

            // Base
            let ul = bits & FLAG_SI_MOSFET_BASE_UL > 0;
            let dr = bits & FLAG_SI_MOSFET_BASE_DR > 0;
            atoms.push(Atom {
                cell_loc,
                si: Placement {
                    center: true,
                    up: is_horizontal && ul,
                    down: is_horizontal && dr,
                    right: !is_horizontal && dr,
                    left: !is_horizontal && ul,
                },
                is_si_n: is_base_n,
                mosfet_part: MosfetPart::Base,
                ..Default::default()
            });
        }
    }

    atoms
}

#[cfg(test)]
mod tests {

    use crate::substrate::{Metal, NormalizedCell, Silicon};

    use super::*;

    #[test]
    fn empty() {
        let ic = IntegratedCircuit::default();
        let bits = serialize_ic(&ic);
        let deserialized_ic = deserialize_ic(&bits);
        assert_eq!(deserialized_ic.chunk_locs().count(), 0);
    }

    #[test]
    fn two_chunks() {
        let mut ic = IntegratedCircuit::default();
        ic.commit_cell_changes(vec![
            (
                IVec2::ONE,
                NormalizedCell {
                    cell_loc: IVec2::ONE,
                    metal: Metal::Trace {
                        has_via: true,
                        placement: Placement::CENTER,
                    },
                    si: Silicon::NP {
                        is_n: true,
                        placement: Placement::CENTER,
                    },
                }
                .into(),
            ),
            (
                -IVec2::ONE,
                NormalizedCell {
                    cell_loc: -IVec2::ONE,
                    metal: Metal::Trace {
                        has_via: true,
                        placement: Placement::CENTER,
                    },
                    si: Silicon::NP {
                        is_n: true,
                        placement: Placement::CENTER,
                    },
                }
                .into(),
            ),
        ]);

        let bits = serialize_ic(&ic);
        let deserialized_ic = deserialize_ic(&bits);
        assert_eq!(deserialized_ic.chunk_locs().count(), 2);
        assert_eq!(
            ic.get_cell_by_location(&IVec2::ONE)
                .cloned()
                .map(NormalizedCell::from),
            deserialized_ic
                .get_cell_by_location(&IVec2::ONE)
                .cloned()
                .map(NormalizedCell::from),
        );
        assert_eq!(
            ic.get_cell_by_location(&(-IVec2::ONE))
                .cloned()
                .map(NormalizedCell::from),
            deserialized_ic
                .get_cell_by_location(&(-IVec2::ONE))
                .cloned()
                .map(NormalizedCell::from),
        );
    }
}
