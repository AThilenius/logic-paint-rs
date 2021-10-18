use std::collections::HashMap;

use glam::{IVec2, Vec2};

use crate::{
    dom::ElementInputEvent,
    substrate::{Cell, IntegratedCircuit, Metal, Silicon},
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
            let to = (steps[i], self.get_cell(&ic, &steps[i]).unwrap_or_default());
            let from = if i > 0 {
                Some((
                    steps[i - 1],
                    self.get_cell(&ic, &steps[i - 1]).unwrap_or_default(),
                ))
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

    fn draw_via(&mut self, from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) {
        // Only draw the first place the user clicks for vias.
        if from.is_none() {
            match (to.1.si, &mut to.1.metal) {
                (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
                _ => {}
            }
        }

        self.cell_overrides.insert(to.0, to.1);
    }

    fn draw_si(
        &mut self,
        ic: &IntegratedCircuit,
        mut from: Option<(IVec2, Cell)>,
        mut to: (IVec2, Cell),
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

        let (tp, tc) = &mut to;

        // We can paint silicon above any cell that doesn't already have silicon.
        if matches!(tc.si, Silicon::None) {
            tc.si = Silicon::NP {
                is_n: paint_n,
                dirs: Default::default(),
                path: 0,
            };
        }

        // Everything else requires a from-cell.
        if let Some((fp, fc)) = &mut from {
            let dir = *tp - *fp;

            // The to cell and tangent cells must all be the opposite type of paint_n to draw a
            // transistor. We check those now to match on it later.
            let tan_dir = IVec2::new(dir.y, dir.x);
            let fc_matches_n = fc.si.matches_n(paint_n);
            let to_cell_transistor_ready = [*tp + tan_dir, *tp, *tp - tan_dir].iter().all(|p| {
                self.get_cell(&ic, p)
                    .map_or(false, |c| c.si.matches_n(!paint_n))
            });
            let next_cell_over_is_not_same_type = self
                .get_cell(&ic, &(*tp + dir))
                .map_or(true, |c| c.si == Silicon::None || c.si.matches_n(paint_n));

            match (&mut fc.si, &mut tc.si) {
                // Single-layer cells of the same type can be joined (painted above).
                (
                    Silicon::NP {
                        is_n: fc_is_n,
                        dirs: fc_dirs,
                        ..
                    },
                    Silicon::NP {
                        is_n: tc_is_n,
                        dirs: tc_dirs,
                        ..
                    },
                ) if fc_is_n == tc_is_n && *tc_is_n == paint_n => {
                    fc_dirs.set_direction(dir, true);
                    tc_dirs.set_direction(-dir, true);
                }
                // An already existing gate can connect with an EMPTY cell in the same direction it's
                // going.
                (
                    Silicon::Mosfet {
                        is_npn, gate_dirs, ..
                    },
                    Silicon::None,
                ) => {
                    gate_dirs.set_direction(dir, true);
                    tc.si = Silicon::NP {
                        is_n: !*is_npn,
                        dirs: (-dir).into(),
                        path: 0,
                    };
                }
                // An already existing gate can connect with an existing single-layer cell in the same
                // direction it's going if that cell is the same type as the gate silicon.
                (
                    Silicon::Mosfet {
                        is_npn, gate_dirs, ..
                    },
                    Silicon::NP { is_n, dirs, .. },
                ) if is_npn != is_n => {
                    gate_dirs.set_direction(dir, true);
                    dirs.set_direction(-dir, true);
                }
                // MOSFETs can only be drawn from silicon, onto single-layer silicon of the opposite
                // type. They also require the tangential cells be of the same type as the MOSFET.
                (
                    Silicon::NP { dirs: fc_dirs, .. }
                    | Silicon::Mosfet {
                        gate_dirs: fc_dirs, ..
                    },
                    Silicon::NP { dirs: tc_dirs, .. },
                ) if fc_matches_n
                    && to_cell_transistor_ready
                    && next_cell_over_is_not_same_type =>
                {
                    fc_dirs.set_direction(dir, true);

                    tc.si = Silicon::Mosfet {
                        is_npn: !paint_n,
                        gate_dirs: (-dir).into(),
                        ec_dirs: *tc_dirs,
                        path: 0,
                    };
                }
                _ => {}
            }
        }

        if let Some((fl, fc)) = from {
            self.cell_overrides.insert(fl, fc);
        }
        self.cell_overrides.insert(to.0, to.1);
    }

    fn draw_metal(&mut self, from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) {
        if let Metal::None = to.1.metal {
            to.1.metal = Metal::Trace {
                has_via: false,
                dirs: Default::default(),
                path: 0,
            };
        }

        if let Some((fd, mut fc)) = from {
            match (&mut fc.metal, &mut to.1.metal) {
                (Metal::Trace { dirs: fc_dirs, .. }, Metal::Trace { dirs: tc_dirs, .. }) => {
                    fc_dirs.set_direction(to.0 - fd, true);
                    tc_dirs.set_direction(fd - to.0, true);
                }
                _ => {}
            }
            self.cell_overrides.insert(fd, fc);
        }
        self.cell_overrides.insert(to.0, to.1);
    }

    pub fn commit_changes(&mut self, ic: &mut IntegratedCircuit) -> bool {
        if !self.previous_buttons.0 && !self.previous_buttons.1 && self.cell_overrides.len() > 0 {
            for (loc, cell) in self.cell_overrides.drain() {
                ic.set_cell(loc, cell);
            }

            return true;
        }

        false
    }

    fn cancel_drawing(&mut self) {
        self.cell_overrides.clear();
    }

    fn get_cell(&self, ic: &IntegratedCircuit, loc: &IVec2) -> Option<Cell> {
        self.cell_overrides.get(loc).cloned().or(ic.get_cell(loc))
    }
}
