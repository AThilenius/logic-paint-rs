use glam::{IVec2, Vec2};

use crate::{
    buffer::Buffer,
    coords::CellCoord,
    substrate::Placement,
    upc::{Metal, NormalizedCell, Silicon},
    utils::range_iter,
    wgl2::Camera,
    RawInput,
};

pub struct Brush {
    active_tool: ToolType,
    draw_start: Option<IVec2>,
    last_event_cell: Option<IVec2>,
    initial_impulse_vertical: Option<bool>,
    previous_buttons: (bool, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    NType,
    PType,
    Metal,
    Via,
}

impl Brush {
    pub fn new() -> Self {
        Self {
            active_tool: ToolType::Metal,
            draw_start: None,
            last_event_cell: None,
            initial_impulse_vertical: None,
            previous_buttons: (false, false),
        }
    }

    pub fn handle_input_event(&mut self, buffer: &mut Buffer, camera: &Camera, event: &RawInput) {
        // Handle keyboard events first
        match &event {
            RawInput::KeyPressed(event) => match event.code().as_ref() {
                "KeyQ" => self.active_tool = ToolType::NType,
                "KeyW" => self.active_tool = ToolType::PType,
                "KeyE" => self.active_tool = ToolType::Metal,
                "KeyR" => self.active_tool = ToolType::Via,
                _ => {}
            },
            _ => {}
        }

        let event = match &event {
            RawInput::MouseDown(e) | RawInput::MouseUp(e) | RawInput::MouseMove(e) => e,
            _ => {
                return;
            }
        };

        let pressed = (event.buttons() & 1 != 0, event.buttons() & 2 != 0);
        let previous = self.previous_buttons;
        let just_pressed = (
            pressed.0 && !self.previous_buttons.0,
            pressed.1 && !self.previous_buttons.1,
        );

        let screen_loc = Vec2::new(event.offset_x() as f32, event.offset_y() as f32);
        let end = camera.project_screen_point_to_cell(screen_loc).0;

        // Only re-compute what was rendered if the mouse moved to another cell.
        if let Some(previous) = self.last_event_cell {
            if previous == end && pressed == self.previous_buttons {
                return;
            }
        }
        self.last_event_cell = Some(end);
        self.previous_buttons = pressed;

        // Clicking both buttons cancels drawing.
        if pressed.0 && pressed.1 {
            buffer.transaction_abort();
            return;
        }

        // If neither button is clicked
        if !pressed.0 && !pressed.1 {
            if previous.0 || previous.1 {
                buffer.transaction_commit(true);
            }

            return;
        }

        // Clear out the old transaction, we repaint everything each time.
        buffer.transaction_abort();
        buffer.transaction_begin();

        if just_pressed.0 || just_pressed.1 {
            self.draw_start = Some(end);
        }

        // Our draw_start should always be set at this point.
        let start = self.draw_start.expect("draw_start should already be set");

        let dist = end - start;

        if dist == IVec2::ZERO {
            self.initial_impulse_vertical = None;
        } else {
            self.initial_impulse_vertical
                .get_or_insert(dist.x.abs() < dist.y.abs());
        }

        // Generate a list of locations affected. This depends on the initial impulse of the user
        // (first direction they move the mouse after clicking).
        let mut steps = vec![];
        if let Some(true) = self.initial_impulse_vertical {
            // Draw Y first, then X.
            for y in range_iter(start.y, end.y) {
                steps.push(CellCoord(IVec2::new(start.x, y)));
            }
            for x in range_iter(start.x, end.x) {
                steps.push(CellCoord(IVec2::new(x, end.y)));
            }
        } else {
            // Draw X first, then Y.
            for x in range_iter(start.x, end.x) {
                steps.push(CellCoord(IVec2::new(x, start.y)));
            }
            for y in range_iter(start.y, end.y) {
                steps.push(CellCoord(IVec2::new(end.x, y)));
            }
        }

        // The last point will be skipped because range_iter is non-inclusive of end point.
        steps.push(CellCoord(end));

        // Draw each step
        for i in 0..steps.len() {
            let to = (steps[i], buffer.get_cell(steps[i]).into());
            let from = if i > 0 {
                Some((steps[i - 1], buffer.get_cell(steps[i - 1]).into()))
            } else {
                None
            };

            match (self.active_tool, pressed.0) {
                (ToolType::NType, true) => self.draw_si(buffer, from, to, true),
                (ToolType::PType, true) => self.draw_si(buffer, from, to, false),
                (ToolType::Metal, true) => self.draw_metal(buffer, from, to),
                (ToolType::Via, true) => self.draw_via(buffer, from, to),
                _ => todo!(),
            }
        }
    }

    fn draw_via(
        &mut self,
        buffer: &mut Buffer,
        from: Option<(CellCoord, NormalizedCell)>,
        (to, mut to_cell): (CellCoord, NormalizedCell),
    ) {
        // Only draw the first place the user clicks for vias.
        if from.is_none() {
            match (to_cell.si, &mut to_cell.metal) {
                (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
                _ => {}
            }
        }

        buffer.transact_set_cell(to, to_cell.into());
    }

    fn draw_si(
        &mut self,
        buffer: &mut Buffer,
        mut from: Option<(CellCoord, NormalizedCell)>,
        (to, mut to_cell): (CellCoord, NormalizedCell),
        paint_n: bool,
    ) {
        // Success cases:
        // - Previous and current don't have silicon
        // - Previous has same type of silicon as to cell
        // - Previous has opposite type of silicon as to cell but also has a gate pointed same direction
        // - Both have same type of silicon (non-op)
        // - No previous, to cell doesn't have si
        // - Previous has same type, to-cell has opposite type and no gate (drawing a gate). Check:
        //   - Adjacent cells are also opposite-type si
        //   - Next cell in same direction != opposite type si

        // We can paint silicon above any cell that doesn't already have silicon.
        if matches!(to_cell.si, Silicon::None) {
            to_cell.si = Silicon::NP {
                is_n: paint_n,
                placement: Placement::CENTER,
            };
        }

        // Everything else requires a from-cell.
        if let Some((from, from_cell)) = &mut from {
            let dir = to.0 - from.0;
            let tan_dir = IVec2::new(dir.y, dir.x);

            // To draw a MOSFET, we need 3 checks:
            // - The cell we are drawing from is the same Si type (to cell will be the Gate)
            let from_cell_matches_n = from_cell.si.matches_n(paint_n);
            // - The 3 tangential cells are the opposite type
            let tangent_cells_match_opposite_n =
                [to.0 + tan_dir, to.0, to.0 - tan_dir].iter().all(|p| {
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
                _ => {}
            }
        }

        if let Some((from, from_cell)) = from {
            buffer.transact_set_cell(from, from_cell.into());
        }
        buffer.transact_set_cell(to, to_cell.into());
    }

    fn draw_metal(
        &mut self,
        buffer: &mut Buffer,
        from: Option<(CellCoord, NormalizedCell)>,
        (to, mut to_cell): (CellCoord, NormalizedCell),
    ) {
        if let Metal::None = to_cell.metal {
            to_cell.metal = Metal::Trace {
                has_via: false,
                placement: Placement::CENTER,
            };
        }

        if let Some((from, mut from_cell)) = from {
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
            buffer.transact_set_cell(from, from_cell.into());
        }
        buffer.transact_set_cell(to, to_cell.into());
    }
}
