use std::collections::HashMap;

use glam::{IVec2, Vec2};

use crate::{
    dom::ElementInputEvent,
    substrate::{Cell, IntegratedCircuit, Metal, NormalizedCell, Placement, Silicon},
    utils::range_iter,
    wgl2::Camera,
};

pub struct Brush {
    pub cell_overrides: HashMap<IVec2, Cell>,
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
            cell_overrides: Default::default(),
            active_tool: ToolType::Metal,
            draw_start: None,
            last_event_cell: None,
            initial_impulse_vertical: None,
            previous_buttons: (false, false),
        }
    }

    pub fn handle_input_event(
        &mut self,
        camera: &Camera,
        ic: &IntegratedCircuit,
        event: ElementInputEvent,
    ) {
        // Handle keyboard events first
        match &event {
            ElementInputEvent::KeyPressed(event) => match event.code().as_ref() {
                "KeyQ" => self.active_tool = ToolType::NType,
                "KeyW" => self.active_tool = ToolType::PType,
                "KeyE" => self.active_tool = ToolType::Metal,
                "KeyR" => self.active_tool = ToolType::Via,
                _ => {}
            },
            _ => {}
        }

        let event = match &event {
            ElementInputEvent::MouseDown(e)
            | ElementInputEvent::MouseUp(e)
            | ElementInputEvent::MouseMove(e) => e,
            _ => {
                return;
            }
        };

        let pressed = (event.buttons() & 1 != 0, event.buttons() & 2 != 0);
        let just_pressed = (
            pressed.0 && !self.previous_buttons.0,
            pressed.1 && !self.previous_buttons.1,
        );

        let screen_loc = Vec2::new(event.offset_x() as f32, event.offset_y() as f32);
        let end = camera.project_screen_point_to_cell(screen_loc);

        if let Some(previous) = self.last_event_cell {
            if previous == end && pressed == self.previous_buttons {
                return;
            }
        }
        self.last_event_cell = Some(end);
        self.previous_buttons = pressed;

        // Clicking both buttons cancels drawing.
        if pressed.0 && pressed.1 {
            self.cancel_drawing();
            return;
        }

        // Nothing to draw if neither button are clicked.
        if !pressed.0 && !pressed.1 {
            return;
        }

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

        // Revert changes first. We will regenerate them.
        self.cell_overrides.clear();

        // Generate a list of locations affected. This depends on the initial impulse of the user
        // (first direction they move the mouse after clicking).
        let mut steps = vec![];
        if let Some(true) = self.initial_impulse_vertical {
            // Draw Y first, then X.
            for y in range_iter(start.y, end.y) {
                steps.push(IVec2::new(start.x, y));
            }
            for x in range_iter(start.x, end.x) {
                steps.push(IVec2::new(x, end.y));
            }
        } else {
            // Draw X first, then Y.
            for x in range_iter(start.x, end.x) {
                steps.push(IVec2::new(x, start.y));
            }
            for y in range_iter(start.y, end.y) {
                steps.push(IVec2::new(end.x, y));
            }
        }

        // The last point will be skipped because range_iter is non-inclusive of end point.
        steps.push(end);

        // Draw each step
        for i in 0..steps.len() {
            let to = self.get_norm_cell_or_default(&ic, &steps[i]);
            let from = if i > 0 {
                Some(self.get_norm_cell_or_default(&ic, &steps[i - 1]))
            } else {
                None
            };

            match (self.active_tool, pressed.0) {
                // (ToolType::None, _) => {}
                (ToolType::NType, true) => self.draw_si(&ic, from, to, true),
                // (ToolType::NType, false) => {}
                (ToolType::PType, true) => self.draw_si(&ic, from, to, false),
                // (ToolType::PType, false) => {}
                (ToolType::Metal, true) => self.draw_metal(from, to),
                // (ToolType::Metal, false) => {}
                (ToolType::Via, true) => self.draw_via(from, to),
                // (ToolType::Via, false) => {}
                _ => todo!(),
            }
        }
    }

    fn draw_via(&mut self, from: Option<NormalizedCell>, mut to: NormalizedCell) {
        // Only draw the first place the user clicks for vias.
        if from.is_none() {
            match (to.si, &mut to.metal) {
                (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
                _ => {}
            }
        }

        self.cell_overrides.insert(to.cell_loc.clone(), to.into());
    }

    fn draw_si(
        &mut self,
        ic: &IntegratedCircuit,
        mut from: Option<NormalizedCell>,
        mut to: NormalizedCell,
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
        if matches!(to.si, Silicon::None) {
            to.si = Silicon::NP {
                is_n: paint_n,
                placement: Placement::CENTER,
            };
        }

        // Everything else requires a from-cell.
        if let Some(from) = &mut from {
            let dir = to.cell_loc - from.cell_loc;

            // The to cell and tangent cells must all be the opposite type of paint_n to draw a
            // transistor. We check those now to match on it later.
            let tan_dir = IVec2::new(dir.y, dir.x);
            let fc_matches_n = from.si.matches_n(paint_n);
            let to_cell_transistor_ready =
                [to.cell_loc + tan_dir, to.cell_loc, to.cell_loc - tan_dir]
                    .iter()
                    .all(|p| {
                        self.get_norm_cell(&ic, p)
                            .map_or(false, |c| c.si.matches_n(!paint_n))
                    });
            let next_cell_over_is_not_same_type = self
                .get_norm_cell(&ic, &(to.cell_loc + dir))
                .map_or(true, |c| c.si == Silicon::None || c.si.matches_n(paint_n));

            match (&mut from.si, &mut to.si) {
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
                    to.si = Silicon::NP {
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
                ) if fc_matches_n
                    && to_cell_transistor_ready
                    && next_cell_over_is_not_same_type =>
                {
                    fc_pl.set_cardinal(dir);
                    let mut gate_placement = Placement::NONE;
                    gate_placement.set_cardinal(-dir);

                    to.si = Silicon::Mosfet {
                        is_npn: !paint_n,
                        gate_placement,
                        ec_placement: *tc_pl,
                    };
                }
                _ => {}
            }
        }

        if let Some(from) = from {
            self.cell_overrides
                .insert(from.cell_loc.clone(), from.into());
        }
        self.cell_overrides.insert(to.cell_loc.clone(), to.into());
    }

    fn draw_metal(&mut self, from: Option<NormalizedCell>, mut to: NormalizedCell) {
        if let Metal::None = to.metal {
            to.metal = Metal::Trace {
                has_via: false,
                placement: Placement::CENTER,
            };
        }

        if let Some(mut from) = from {
            match (&mut from.metal, &mut to.metal) {
                (
                    Metal::Trace {
                        placement: fc_pl, ..
                    },
                    Metal::Trace {
                        placement: tc_pl, ..
                    },
                ) => {
                    fc_pl.set_cardinal(to.cell_loc - from.cell_loc);
                    tc_pl.set_cardinal(from.cell_loc - to.cell_loc);
                }
                _ => {}
            }
            self.cell_overrides
                .insert(from.cell_loc.clone(), from.into());
        }
        self.cell_overrides.insert(to.cell_loc.clone(), to.into());
    }

    pub fn commit_changes(&mut self, ic: &mut IntegratedCircuit) -> bool {
        if !self.previous_buttons.0 && !self.previous_buttons.1 && self.cell_overrides.len() > 0 {
            ic.commit_cell_changes(self.cell_overrides.drain().collect());
            return true;
        }

        false
    }

    fn cancel_drawing(&mut self) {
        self.cell_overrides.clear();
    }

    fn get_norm_cell(&self, ic: &IntegratedCircuit, loc: &IVec2) -> Option<NormalizedCell> {
        self.cell_overrides
            .get(loc)
            .cloned()
            .or(ic.get_cell_by_location(loc).cloned())
            .map(|c| c.into())
    }

    fn get_norm_cell_or_default(&self, ic: &IntegratedCircuit, loc: &IVec2) -> NormalizedCell {
        let mut cell: NormalizedCell = self
            .cell_overrides
            .get(loc)
            .cloned()
            .or(ic.get_cell_by_location(loc).cloned())
            .map(|c| c.into())
            .unwrap_or_default();

        cell.cell_loc = *loc;

        cell
    }
}
