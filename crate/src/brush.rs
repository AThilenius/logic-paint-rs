use glam::IVec2;

use crate::{
    buffer::Buffer,
    coords::CellCoord,
    upc::{Metal, NormalizedCell, Placement, Silicon},
};

pub fn draw_via(buffer: &mut Buffer, from: Option<CellCoord>, to: CellCoord) {
    // Only draw the first place the user clicks for vias.
    let mut to_cell: NormalizedCell = buffer.get_cell(to).into();

    if from.is_none() {
        match (to_cell.si, &mut to_cell.metal) {
            (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
            _ => {}
        }
    }

    buffer.set_cell(to, to_cell.into());
}

pub fn draw_si(buffer: &mut Buffer, from: Option<CellCoord>, to: CellCoord, paint_n: bool) {
    // Success cases:
    // - Previous and current don't have silicon
    // - Previous has same type of silicon as to cell
    // - Previous has opposite type of silicon as to cell but also has a gate pointed same direction
    // - Both have same type of silicon (non-op)
    // - No previous, to cell doesn't have si
    // - Previous has same type, to-cell has opposite type and no gate (drawing a gate). Check:
    //   - Adjacent cells are also opposite-type si
    //   - Next cell in same direction != opposite type si
    let mut to_cell: NormalizedCell = buffer.get_cell(to).into();

    // We can paint silicon above any cell that doesn't already have silicon.
    if matches!(to_cell.si, Silicon::None) {
        to_cell.si = Silicon::NP {
            is_n: paint_n,
            placement: Placement::CENTER,
        };
    }

    let mut from = from.map(|coord| (coord.clone(), NormalizedCell::from(buffer.get_cell(coord))));

    // Everything else requires a from-cell.
    if let Some((from, from_cell)) = &mut from {
        let dir = to.0 - from.0;
        let tan_dir = IVec2::new(dir.y, dir.x);

        // To draw a MOSFET, we need 4 checks, but one of those checks is implicit in the
        // match statement below (that the to-be MOSFET isn't already a MOSFET).
        // - The cell we are drawing from is the same Si type (to cell will be the Gate)
        let from_cell_matches_n = from_cell.si.matches_n(paint_n);
        // - There is a line of 3 silicon of the opposite type of us (or gates there of)
        //   tangential to th gate.
        let dirs = [to.0 + tan_dir, to.0, to.0 - tan_dir];
        let tangent_cells_match_opposite_n = dirs.iter().all(|p| {
            NormalizedCell::from(buffer.get_cell(CellCoord(*p)))
                .si
                .matches_n(!paint_n)
        });
        // - The Si under the to-be Gate doesn't connect in the direction we are going.
        let to_cell_does_not_connect_in_dir = {
            if let Silicon::NP { placement, .. } = to_cell.si {
                !placement.cardinal_vectors().contains(&dir)
            } else {
                false
            }
        };

        match (&mut from_cell.si, &mut to_cell.si) {
            // Single-layer cells of the same type can be joined (painted above).
            (
                Silicon::NP {
                    is_n: fc_is_n,
                    placement: fc_pl,
                    ..
                },
                Silicon::NP {
                    is_n: tc_is_n,
                    placement: tc_pl,
                    ..
                },
            ) if fc_is_n == tc_is_n && *tc_is_n == paint_n => {
                fc_pl.set_cardinal(dir);
                tc_pl.set_cardinal(-dir);
            }
            // An already existing gate can connect with an EMPTY cell in the same direction it's
            // going.
            (
                Silicon::Mosfet {
                    is_npn,
                    gate_placement,
                    ..
                },
                Silicon::None,
            ) => {
                gate_placement.set_cardinal(dir);
                let mut placement = Placement::NONE;
                placement.set_cardinal(-dir);
                to_cell.si = Silicon::NP {
                    is_n: !*is_npn,
                    placement,
                };
            }
            // An already existing gate can connect with an existing single-layer cell in the same
            // direction it's going if that cell is the same type as the gate silicon.
            (
                Silicon::Mosfet {
                    is_npn,
                    gate_placement,
                    ..
                },
                Silicon::NP {
                    is_n, placement, ..
                },
            ) if is_npn != is_n => {
                gate_placement.set_cardinal(dir);
                placement.set_cardinal(-dir);
            }
            // MOSFETs can only be drawn from silicon, onto single-layer silicon of the opposite
            // type. They also require the tangential cells be of the same type as the MOSFET.
            (
                Silicon::NP {
                    placement: fc_pl, ..
                }
                | Silicon::Mosfet {
                    gate_placement: fc_pl,
                    ..
                },
                Silicon::NP {
                    placement: tc_pl, ..
                },
            ) if from_cell_matches_n
                && tangent_cells_match_opposite_n
                && to_cell_does_not_connect_in_dir =>
            {
                fc_pl.set_cardinal(dir);
                let mut gate_placement = Placement::NONE;
                gate_placement.set_cardinal(-dir);

                to_cell.si = Silicon::Mosfet {
                    is_npn: !paint_n,
                    gate_placement,
                    ec_placement: *tc_pl,
                };
            }
            // You can draw from an NP onto a MOSFET's gate.
            (
                Silicon::NP {
                    is_n, placement, ..
                },
                Silicon::Mosfet {
                    is_npn,
                    gate_placement,
                    ..
                },
            ) if *is_n == paint_n && is_n != is_npn && gate_placement.has_cardinal(dir) => {
                placement.set_cardinal(dir);
                gate_placement.set_cardinal(-dir);
            }
            // Or from an MOSFET gate onto another MOSFET's gate.
            (
                Silicon::Mosfet {
                    is_npn: fc_npn,
                    gate_placement: fc_gp,
                    ..
                },
                Silicon::Mosfet {
                    is_npn: tc_npn,
                    gate_placement: tc_gp,
                    ..
                },
            ) if *fc_npn != paint_n
                && fc_npn == tc_npn
                && fc_gp.has_cardinal(dir)
                && tc_gp.has_cardinal(-dir) =>
            {
                tc_gp.set_cardinal(dir);
                fc_gp.set_cardinal(-dir);
            }

            _ => {}
        }
    }

    if let Some((from, from_cell)) = from {
        buffer.set_cell(from, from_cell.into());
    }

    buffer.set_cell(to, to_cell.into());
}

pub fn draw_metal(buffer: &mut Buffer, from: Option<CellCoord>, to: CellCoord) {
    let mut to_cell: NormalizedCell = buffer.get_cell(to).into();

    if let Metal::None = to_cell.metal {
        to_cell.metal = Metal::Trace {
            has_via: false,
            placement: Placement::CENTER,
        };
    }

    if let Some(from) = from {
        let mut from_cell: NormalizedCell = buffer.get_cell(from).into();
        match (&mut from_cell.metal, &mut to_cell.metal) {
            (
                Metal::Trace {
                    placement: fc_pl, ..
                },
                Metal::Trace {
                    placement: tc_pl, ..
                },
            ) => {
                fc_pl.set_cardinal(to.0 - from.0);
                tc_pl.set_cardinal(from.0 - to.0);
            }
            _ => {}
        }
        buffer.set_cell(from, from_cell.into());
    }
    buffer.set_cell(to, to_cell.into());
}
