use arrayvec::ArrayVec;
use glam::IVec2;
use wasm_bindgen::prelude::*;

use crate::{
    coords::CellCoord,
    substrate::buffer::Buffer,
    upc::{Metal, NormalizedCell, Placement, Silicon},
    utils::{range_iter, Selection},
};

pub type Path = Vec<CellCoord>;

#[wasm_bindgen]
impl Buffer {
    pub fn draw_si(
        &mut self,
        CellCoord(start): CellCoord,
        CellCoord(end): CellCoord,
        initial_impulse_vertical: bool,
        paint_n: bool,
    ) {
        let mut from = None;
        if initial_impulse_vertical {
            // Draw Y first, then X.
            for y in range_iter(start.y, end.y) {
                self.draw_si_link(from, (start.x, y).into(), paint_n);
                from = Some((start.x, y).into());
            }
            for x in range_iter(start.x, end.x) {
                self.draw_si_link(from, (x, end.y).into(), paint_n);
                from = Some((x, end.y).into());
            }
        } else {
            // Draw X first, then Y.
            for x in range_iter(start.x, end.x) {
                self.draw_si_link(from, (x, start.y).into(), paint_n);
                from = Some((x, start.y).into());
            }
            for y in range_iter(start.y, end.y) {
                self.draw_si_link(from, (end.x, y).into(), paint_n);
                from = Some((end.x, y).into());
            }
        }

        self.draw_si_link(from, (end.x, end.y).into(), paint_n);
    }

    pub fn draw_metal(
        &mut self,
        CellCoord(start): CellCoord,
        CellCoord(end): CellCoord,
        initial_impulse_vertical: bool,
    ) {
        let mut from = None;
        if initial_impulse_vertical {
            // Draw Y first, then X.
            for y in range_iter(start.y, end.y) {
                self.draw_metal_link(from, (start.x, y).into());
                from = Some((start.x, y).into());
            }
            for x in range_iter(start.x, end.x) {
                self.draw_metal_link(from, (x, end.y).into());
                from = Some((x, end.y).into());
            }
        } else {
            // Draw X first, then Y.
            for x in range_iter(start.x, end.x) {
                self.draw_metal_link(from, (x, start.y).into());
                from = Some((x, start.y).into());
            }
            for y in range_iter(start.y, end.y) {
                self.draw_metal_link(from, (end.x, y).into());
                from = Some((end.x, y).into());
            }
        }

        self.draw_metal_link(from, (end.x, end.y).into());
    }

    pub fn draw_via(&mut self, cell_coord: CellCoord) {
        // Only draw the first place the user clicks for vias.
        let mut to_cell: NormalizedCell = self.get_cell(cell_coord).into();

        match (to_cell.si, &mut to_cell.metal) {
            (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
            _ => {}
        }

        self.set_cell(cell_coord, to_cell.into());
    }

    pub fn clear_selection(&mut self, selection: &Selection) {
        if selection.is_zero() {
            return;
        }

        let ll = selection.lower_left.0;
        let ur = selection.upper_right.0;

        self.clear_selection_border(selection);

        // Then we can just blit-clear the inside.
        for y in (ll.y + 1)..(ur.y - 1) {
            for x in (ll.x + 1)..(ur.x - 1) {
                self.set_cell(CellCoord(IVec2::new(x, y)), Default::default());
            }
        }
    }

    pub fn clear_selection_border(&mut self, selection: &Selection) {
        if selection.is_zero() {
            return;
        }

        let ll = selection.lower_left.0;
        let ur = selection.upper_right.0;

        (ll.x..ur.x).for_each(|x| self.clear_cell_si((x, ll.y).into()));
        (ll.x..ur.x).for_each(|x| self.clear_cell_si((x, ur.y - 1).into()));
        (ll.y..ur.y).for_each(|y| self.clear_cell_si((ll.x, y).into()));
        (ll.y..ur.y).for_each(|y| self.clear_cell_si((ur.x - 1, y).into()));

        (ll.x..ur.x).for_each(|x| self.clear_cell_metal((x, ll.y).into()));
        (ll.x..ur.x).for_each(|x| self.clear_cell_metal((x, ur.y - 1).into()));
        (ll.y..ur.y).for_each(|y| self.clear_cell_metal((ll.x, y).into()));
        (ll.y..ur.y).for_each(|y| self.clear_cell_metal((ur.x - 1, y).into()));
    }

    pub fn draw_si_link(&mut self, from: Option<CellCoord>, to: CellCoord, paint_n: bool) {
        let mut to_cell: NormalizedCell = self.get_cell(to).into();

        // We can paint silicon above any cell that doesn't already have silicon.
        if matches!(to_cell.si, Silicon::None) {
            to_cell.si = Silicon::NP {
                is_n: paint_n,
                placement: Placement::CENTER,
            };

            self.set_cell(to, to_cell.into());
        }

        // Everything else requires a from-cell.
        if let Some(from) = from {
            let from_cell = NormalizedCell::from(self.get_cell(from));

            let dir = to.0 - from.0;

            // If the from_cell is already connected, then there is nothing to do.
            match from_cell.si {
                Silicon::NP { placement, .. } if placement.has_cardinal(dir) => return,
                Silicon::Mosfet {
                    gate_placement,
                    ec_placement,
                    ..
                } if gate_placement.has_cardinal(dir) || ec_placement.has_cardinal(dir) => return,
                _ => {}
            }

            // At this point both from-cell and to-cell will both have silicon. We can take an easy
            // short-circuit by looking at if the from cell can possibly connect given what type we are
            // painting, and what direction we are going. We can connect when:
            // N : NP(n) : connect
            // P : NP(p) : connect
            // N : MOSFET(npn) && ec_in_line_with_dir : connect ec
            // N : MOSFET(pnp) && gate_in_line_with_dir : connect gate
            // P : MOSFET(npn) && gate_in_line_with_dir : connect gate
            // P : MOSFET(pnp) && ec_in_line_with_dir : connect ec
            //
            // No other connections can possibly connect, so we can short-circuit. If we don't short
            // circuit, then we can assume the si we are painting is what is going to be drawn and
            // connected. We can optimistically create the connection from the from-cell (in an
            // temporary clone) before going on to the to-cell.

            let mut connected_from_cell = from_cell.clone();
            let going_horizontal = dir.x != 0;

            match (paint_n, &mut connected_from_cell.si) {
                (
                    true,
                    Silicon::NP {
                        is_n: true,
                        placement,
                    },
                ) => {
                    placement.set_cardinal(dir);
                }
                (
                    false,
                    Silicon::NP {
                        is_n: false,
                        placement,
                    },
                ) => {
                    placement.set_cardinal(dir);
                }
                (
                    true,
                    Silicon::Mosfet {
                        is_npn: true,
                        is_horizontal,
                        ec_placement,
                        ..
                    },
                ) if *is_horizontal != going_horizontal => {
                    ec_placement.set_cardinal(dir);
                }
                (
                    true,
                    Silicon::Mosfet {
                        is_npn: false,
                        is_horizontal,
                        gate_placement,
                        ..
                    },
                ) if *is_horizontal == going_horizontal => {
                    gate_placement.set_cardinal(dir);
                }
                (
                    false,
                    Silicon::Mosfet {
                        is_npn: true,
                        is_horizontal,
                        gate_placement,
                        ..
                    },
                ) if *is_horizontal == going_horizontal => {
                    gate_placement.set_cardinal(dir);
                }
                (
                    false,
                    Silicon::Mosfet {
                        is_npn: false,
                        is_horizontal,
                        ec_placement,
                        ..
                    },
                ) if *is_horizontal != going_horizontal => {
                    ec_placement.set_cardinal(dir);
                }
                _ => {}
            }

            // If the from-cell can't connect, then we can skip the rest.
            if connected_from_cell == from_cell {
                return;
            }

            // Now we have to test the to-cell to see if we can paint it. That is possible when:
            // N : NP(n) : connect
            // P : NP(p) : connect
            // N : NP(p) && doesnt_connect_in_dir : make MOSFET(PNP), connect gate
            // P : NP(n) && doesnt_connect_in_dir : make MOSFET(NPN), connect gate
            // N : MOSFET(npn) && ec_in_line_with_dir : connect ec
            // P : MOSFET(npn) && gate_in_line_with_dir : connect gate
            // N : MOSFET(pnp) && gate_in_line_with_dir : connect gate
            // P : MOSFET(pnp) && ec_in_line_with_dir : connect ec
            //
            // No other connections can possibly be made. We can throw away the from-cell connection.
            // Otherwise we commit the from-cell change.
            match (paint_n, &mut to_cell.si) {
                (
                    true,
                    Silicon::NP {
                        is_n: true,
                        placement,
                    },
                ) => {
                    placement.set_cardinal(-dir);
                }
                (
                    false,
                    Silicon::NP {
                        is_n: false,
                        placement,
                    },
                ) => {
                    placement.set_cardinal(-dir);
                }
                (
                    true,
                    Silicon::NP {
                        is_n: false,
                        placement,
                    },
                ) if !placement.has_cardinal(dir) => {
                    to_cell.si = Silicon::Mosfet {
                        is_npn: false,
                        is_horizontal: going_horizontal,
                        gate_placement: Placement::from_cardinal(-dir),
                        ec_placement: *placement,
                    }
                }
                (
                    false,
                    Silicon::NP {
                        is_n: true,
                        placement,
                    },
                ) if !placement.has_cardinal(dir) => {
                    to_cell.si = Silicon::Mosfet {
                        is_npn: true,
                        is_horizontal: going_horizontal,
                        gate_placement: Placement::from_cardinal(-dir),
                        ec_placement: *placement,
                    }
                }
                (
                    true,
                    Silicon::Mosfet {
                        is_npn: true,
                        is_horizontal,
                        ec_placement,
                        ..
                    },
                ) if *is_horizontal != going_horizontal => {
                    ec_placement.set_cardinal(-dir);
                }
                (
                    false,
                    Silicon::Mosfet {
                        is_npn: true,
                        is_horizontal,
                        gate_placement,
                        ..
                    },
                ) if *is_horizontal == going_horizontal => {
                    gate_placement.set_cardinal(-dir);
                }

                (
                    true,
                    Silicon::Mosfet {
                        is_npn: false,
                        is_horizontal,
                        gate_placement,
                        ..
                    },
                ) if *is_horizontal == going_horizontal => {
                    gate_placement.set_cardinal(-dir);
                }
                (
                    false,
                    Silicon::Mosfet {
                        is_npn: false,
                        is_horizontal,
                        ec_placement,
                        ..
                    },
                ) if *is_horizontal != going_horizontal => {
                    ec_placement.set_cardinal(-dir);
                }
                _ => {
                    // Can't connect to to_cell, so abort drawing.
                    return;
                }
            }

            // If we made it to here then both from and to connected up, so we can write both into the
            // buffer.
            self.set_cell(from, connected_from_cell.into());
            self.set_cell(to, to_cell.into());
        }
    }

    pub fn draw_metal_link(&mut self, from: Option<CellCoord>, to: CellCoord) {
        let mut to_cell: NormalizedCell = self.get_cell(to).into();

        if let Metal::None = to_cell.metal {
            to_cell.metal = Metal::Trace {
                has_via: false,
                placement: Placement::CENTER,
            };
        }

        if let Some(from) = from {
            let mut from_cell: NormalizedCell = self.get_cell(from).into();
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
            self.set_cell(from, from_cell.into());
        }
        self.set_cell(to, to_cell.into());
    }

    pub fn clear_cell_si(&mut self, cell_coord: CellCoord) {
        let upc = self.get_cell(cell_coord);

        if upc == Default::default() {
            return;
        }

        let mut normalized: NormalizedCell = self.get_cell(cell_coord.clone()).into();

        let vectors = match normalized.si {
            Silicon::NP { placement, .. } => placement.cardinal_vectors(),
            Silicon::Mosfet {
                gate_placement,
                ec_placement,
                ..
            } => (gate_placement | ec_placement).cardinal_vectors(),
            _ => ArrayVec::new(),
        };

        // Clear the target cell
        normalized.si = Silicon::None;

        // Clear via as well
        match &mut normalized.metal {
            Metal::Trace { has_via, .. } => *has_via = false,
            _ => {}
        }

        self.set_cell(cell_coord, normalized.into());

        // Then un-link neighbors.
        for vector in vectors {
            let neighbor_coord = CellCoord(cell_coord.0 + vector);
            let mut neighbor: NormalizedCell = self.get_cell(neighbor_coord).into();

            match &mut neighbor.si {
                Silicon::NP { placement, .. } => placement.clear_cardinal(-vector),
                Silicon::Mosfet {
                    gate_placement,
                    ec_placement,
                    ..
                } => {
                    gate_placement.clear_cardinal(-vector);
                    ec_placement.clear_cardinal(-vector);
                }
                _ => {}
            }

            self.set_cell(neighbor_coord, neighbor.into());
        }
    }

    pub fn clear_cell_metal(&mut self, cell_coord: CellCoord) {
        let upc = self.get_cell(cell_coord);

        if upc == Default::default() {
            return;
        }

        let mut normalized: NormalizedCell = self.get_cell(cell_coord.clone()).into();

        let vectors = match normalized.metal {
            Metal::Trace { placement, .. } => placement.cardinal_vectors(),
            _ => ArrayVec::new(),
        };

        // Clear the target cell
        normalized.metal = Metal::None;
        self.set_cell(cell_coord, normalized.into());

        // Then un-link neighbors.
        for vector in vectors {
            let neighbor_coord = CellCoord(cell_coord.0 + vector);
            let mut neighbor: NormalizedCell = self.get_cell(neighbor_coord).into();

            match &mut neighbor.metal {
                Metal::Trace { placement, .. } => placement.clear_cardinal(-vector),
                _ => {}
            }

            self.set_cell(neighbor_coord, neighbor.into());
        }
    }
}
