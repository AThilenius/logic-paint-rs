use crate::{
    canvas::Canvas,
    sim::Graph,
    utils::{range_iter, unwrap::unwrap_or_return},
};
use bevy::prelude::*;

use super::{
    input::{ActiveTools, CanvasInput, ToolType},
    Cell, Metal, Silicon,
};

#[derive(Default)]
pub struct CanvasDrawing {
    pub draw_start: Option<IVec2>,
    pub changes: Vec<CanvasCellChange>,
    pub initial_impulse: Option<InitialImpulse>,
}

pub enum InitialImpulse {
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanvasCellChange {
    pub location: IVec2,
    pub from_cell: Cell,
}

impl CanvasDrawing {
    pub fn revert_changes(&mut self, data: &mut Canvas) {
        for change in self.changes.iter() {
            if let Some(cell) = data.cells.get_mut(&change.location) {
                *cell = change.from_cell;
            }
        }
        self.changes.clear();
    }
}

pub fn handle_canvas_input(
    mut canvas_query: Query<(&mut Canvas, &mut CanvasDrawing, &CanvasInput)>,
    active_tool: Res<ActiveTools>,
) {
    for (mut canvas, mut drawing, input) in canvas_query.iter_mut() {
        // DEV
        if input.compile_just_clicked {
            let network = canvas.compile_to_network();
            let graph: Graph = network.into();
            println!("{}", graph);
        }

        if input.left_pressed && input.right_pressed {
            // Cancel the draw
            drawing.revert_changes(&mut canvas);
            drawing.draw_start = None;
            return;
        }

        if !input.left_pressed && !input.right_pressed {
            // Changes are already committed, so we just clear out the draw_start and changed.
            drawing.draw_start = None;
            drawing.changes.clear();
            return;
        }

        if input.left_just_pressed || input.right_just_pressed {
            drawing.draw_start = input.mouse_position;
        }

        // This shouldn't technically be possible as the input system should filter clicks that
        // don't start on a canvas. But good safety check.
        let start = unwrap_or_return!(drawing.draw_start);

        // The mouse was moved off the canvas while drawing. We simply ignore it, leaving the old
        // set of changes as they were.
        let end = unwrap_or_return!(input.mouse_position);

        let dist = end - start;

        if start == end {
            drawing.initial_impulse = None;
        } else if drawing.initial_impulse.is_none() {
            drawing.initial_impulse = Some(if dist.x.abs() > dist.y.abs() {
                InitialImpulse::X
            } else {
                InitialImpulse::Y
            });
        }

        // Revert changes first. We will regenerate them.
        drawing.revert_changes(&mut canvas);

        // Generate a list of locations affected. This depends on the initial impulse of the user
        // (first direction they move the mouse after clicking).
        let mut steps = vec![];
        if let Some(InitialImpulse::Y) = drawing.initial_impulse {
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

        // The last point will be skipped because iter is non-inclusive of end point.
        steps.push(end);

        // Draw each step
        for i in 0..steps.len() {
            let to = canvas.cells.get(&steps[i]).cloned().unwrap_or_default();

            // Save the cell's original state for reverting.
            drawing.changes.push(CanvasCellChange {
                location: steps[i],
                from_cell: to.clone(),
            });

            let from = if i > 0 {
                Some((
                    steps[i - 1],
                    *canvas
                        .cells
                        .get(&steps[i - 1])
                        .unwrap_or(&Default::default()),
                ))
            } else {
                None
            };

            let (prev, new) = draw(
                &canvas,
                &active_tool,
                input.left_pressed,
                from,
                (steps[i], to),
            );

            // Write both cells to the canvas data.
            if let Some(prev) = prev {
                canvas.cells.insert(steps[i - 1], prev);
            }
            canvas.cells.insert(steps[i], new);
        }
    }
}

fn draw(
    data: &Canvas,
    active_tool: &ActiveTools,
    left_click: bool,
    from: Option<(IVec2, Cell)>,
    to: (IVec2, Cell),
) -> (Option<Cell>, Cell) {
    match (active_tool.tool_type, left_click) {
        // (ToolType::None, _) => {}
        (ToolType::NType, true) => draw_si(data, from, to, true),
        // (ToolType::NType, false) => {}
        (ToolType::PType, true) => draw_si(data, from, to, false),
        // (ToolType::PType, false) => {}
        (ToolType::Metal, true) => draw_metal(from, to),
        // (ToolType::Metal, false) => {}
        (ToolType::Via, true) => draw_via(from, to),
        // (ToolType::Via, false) => {}
        _ => todo!(),
    }
}

fn draw_via(from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) -> (Option<Cell>, Cell) {
    // Only draw the first place the user clicks for vias.
    if from.is_none() {
        match (to.1.si, &mut to.1.metal) {
            (Silicon::NP { .. }, Metal::Trace { has_via, .. }) => *has_via = true,
            _ => {}
        }
    }

    (from.map(|f| f.1), to.1)
}

fn draw_si(
    data: &Canvas,
    mut from: Option<(IVec2, Cell)>,
    mut to: (IVec2, Cell),
    paint_n: bool,
) -> (Option<Cell>, Cell) {
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

    // We can paint silicon above any non-IO pin that doesn't already have silicon.
    if matches!(tc.si, Silicon::None) && !matches!(tc.metal, Metal::IO { .. }) {
        tc.si = Silicon::NP {
            is_n: paint_n,
            dirs: Default::default(),
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
            data.cells
                .get(p)
                .map_or(false, |c| c.si.matches_n(!paint_n))
        });
        let next_cell_over_is_not_same_type = data
            .cells
            .get(&(*tp + dir))
            .map_or(false, |c| c.si == Silicon::None || c.si.matches_n(paint_n));

        match (&mut fc.si, &mut tc.si) {
            // Single-layer cells of the same type can be joined (painted above).
            (
                Silicon::NP {
                    is_n: fc_is_n,
                    dirs: fc_dirs,
                },
                Silicon::NP {
                    is_n: tc_is_n,
                    dirs: tc_dirs,
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
                };
            }
            // An already existing gate can connect with an existing single-layer cell in the same
            // direction it's going if that cell is the same type as the gate silicon.
            (
                Silicon::Mosfet {
                    is_npn, gate_dirs, ..
                },
                Silicon::NP { is_n, dirs },
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
            ) if fc_matches_n && to_cell_transistor_ready && next_cell_over_is_not_same_type => {
                fc_dirs.set_direction(dir, true);

                tc.si = Silicon::Mosfet {
                    is_npn: !paint_n,
                    gate_dirs: (-dir).into(),
                    ec_dirs: *tc_dirs,
                };
            }
            _ => {}
        }
    }

    (from.map(|f| f.1), to.1)
}

fn draw_metal(from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) -> (Option<Cell>, Cell) {
    if let Metal::None = to.1.metal {
        to.1.metal = Metal::Trace {
            has_via: false,
            dirs: Default::default(),
        };
    }

    if let Some((fd, mut fc)) = from {
        match (&mut fc.metal, &mut to.1.metal) {
            (
                Metal::Trace { dirs: fc_dirs, .. } | Metal::IO { dirs: fc_dirs },
                Metal::Trace { dirs: tc_dirs, .. } | Metal::IO { dirs: tc_dirs },
            ) => {
                fc_dirs.set_direction(to.0 - fd, true);
                tc_dirs.set_direction(fd - to.0, true);
            }
            _ => {}
        }
        (Some(fc), to.1)
    } else {
        (None, to.1)
    }
}
