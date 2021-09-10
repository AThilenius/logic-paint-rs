use crate::{
    canvas::CanvasData,
    utils::{range_iter, unwrap::unwrap_or_return},
};
use bevy::prelude::*;

use super::{
    input::{ActiveTools, CanvasInput, ToolType},
    Cell,
};

#[derive(Default)]
pub struct CanvasDrawing {
    pub draw_start: Option<IVec2>,
    pub changes: Vec<CanvasCellChange>,
    pub initial_inpulse: Option<InitialInpulse>,
}

pub enum InitialInpulse {
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanvasCellChange {
    pub location: IVec2,
    pub from_cell: Cell,
}

impl CanvasDrawing {
    pub fn revert_changes(&mut self, data: &mut CanvasData) {
        for change in self.changes.iter() {
            *data.get_cell_mut(change.location) = change.from_cell;
        }
        self.changes.clear();
    }
}

pub fn handle_canvas_input(
    mut canvas_query: Query<(&mut CanvasData, &mut CanvasDrawing, &CanvasInput)>,
    active_tool: Res<ActiveTools>,
) {
    for (mut data, mut drawing, input) in canvas_query.iter_mut() {
        if input.left_pressed && input.right_pressed {
            // Cancel the draw
            drawing.revert_changes(&mut data);
            drawing.draw_start = None;
            return;
        }

        if !input.left_pressed && !input.right_pressed {
            // Changes are already commited, so we just clear out the draw_start and changed.
            drawing.draw_start = None;
            drawing.changes.clear();
            return;
        }

        if input.left_just_pressed || input.right_just_pressed {
            drawing.draw_start = input.mouse_position;
        }

        // This shouldn't technically be possible as the input system should filter clicks that
        // don't start on a canvas. But good satefy check.
        let start = unwrap_or_return!(drawing.draw_start);

        // The mouse was moved off the canvas while drawing. We simply ignore it, leaving the old
        // set of changes as they were.
        let end = unwrap_or_return!(input.mouse_position);

        let dist = end - start;

        if start == end {
            drawing.initial_inpulse = None;
        } else if drawing.initial_inpulse.is_none() {
            drawing.initial_inpulse = Some(if dist.x.abs() > dist.y.abs() {
                InitialInpulse::X
            } else {
                InitialInpulse::Y
            });
        }

        // Revert changes first. We will regenerage them.
        drawing.revert_changes(&mut data);

        // Generate a list of locations affected. This depends on the initial inpulse of the user
        // (first direction they move the mouse after clicking).
        let mut steps = vec![];
        if let Some(InitialInpulse::Y) = drawing.initial_inpulse {
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
            let to = data.get_cell(steps[i]).clone();

            // Save the cell's original state for reverting.
            drawing.changes.push(CanvasCellChange {
                location: steps[i],
                from_cell: to.clone(),
            });

            let from = if i > 0 {
                Some((steps[i - 1], data.get_cell(steps[i - 1]).clone()))
            } else {
                None
            };

            let (prev, new) = draw(
                &data,
                &active_tool,
                input.left_pressed,
                from,
                (steps[i], to),
            );

            // Write both cells to the canvas data.
            if let Some(prev) = prev {
                *data.get_cell_mut(steps[i - 1]) = prev;
            }
            *data.get_cell_mut(steps[i]) = new;
        }
    }
}

fn draw(
    data: &CanvasData,
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

fn draw_via(mut from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) -> (Option<Cell>, Cell) {
    // Only draw the first place the user clicks for vias.
    let (_, tc) = &mut to;
    if from.is_none() && (tc.si_n || tc.si_p) && tc.metal {
        to.1.via = true;
    }

    (from.map(|f| f.1), to.1)
}

fn draw_si(
    data: &CanvasData,
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

    // We can always paint silicon if the current cell has none. We might not join it up though.
    if !tc.si_n && !tc.si_p {
        tc.si_n = paint_n;
        tc.si_p = !paint_n;
    }

    // Everything else requires a from-cell.
    if let Some((fp, fc)) = &mut from {
        let dir = *tp - *fp;

        // If the cells are both what we are paining, we can always join them.
        if fc.si_n == tc.si_n && tc.si_n == paint_n && fc.si_p == tc.si_p && tc.si_p != paint_n {
            fc.si_dirs.set_direction(*tp - *fp, true);
            tc.si_dirs.set_direction(*fp - *tp, true);
        }

        // If the from-cell is the opposite type, but has a gate going in the same direction.
        if fc.si_n != tc.si_n
            && fc.si_p != tc.si_p
            && tc.si_n == paint_n
            && tc.si_p != paint_n
            && fc.gate_dirs.matches_gate_direction(dir)
        {
            fc.gate_dirs.set_direction(*tp - *fp, true);
            tc.si_dirs.set_direction(*fp - *tp, true);
        }

        // If the to-cell has the opposite type as paint, and...
        // - Doesn't have a gate or has a gate in the same direction
        // - From-cell has same type as paint
        // - Left-Right orientation:
        //   - Up-Down both have same si as to-cell
        // - Up-Down is the inverse of Left-Right
        let is_to_opposite_paint = tc.si_n != paint_n && tc.si_p == paint_n;
        let is_from_same_as_paint = fc.si_n == paint_n && fc.si_p == !paint_n;
        let is_dir_none_or_matching =
            tc.gate_dirs.is_none() || tc.gate_dirs.matches_gate_direction(dir);
        let is_left_right = dir.x.abs() != 0;
        let is_tangential_cells_same_as_to_cell_or_none = if is_left_right {
            data.get_cell_checked(*tp + IVec2::Y)
                .map_or(true, |c| c.si_n != paint_n && c.si_p == paint_n)
                && data
                    .get_cell_checked(*tp - IVec2::Y)
                    .map_or(true, |c| c.si_n != paint_n && c.si_p == paint_n)
        } else {
            data.get_cell_checked(*tp + IVec2::X)
                .map_or(true, |c| c.si_n != paint_n && c.si_p == paint_n)
                && data
                    .get_cell_checked(*tp - IVec2::X)
                    .map_or(true, |c| c.si_n != paint_n && c.si_p == paint_n)
        };

        if is_to_opposite_paint
            && is_from_same_as_paint
            && is_dir_none_or_matching
            && is_tangential_cells_same_as_to_cell_or_none
        {
            fc.si_dirs.set_direction(*tp - *fp, true);
            tc.gate_dirs.set_direction(*fp - *tp, true);
        }
    }

    (from.map(|f| f.1), to.1)
}

fn draw_metal(from: Option<(IVec2, Cell)>, mut to: (IVec2, Cell)) -> (Option<Cell>, Cell) {
    to.1.metal = true;

    if let Some(mut from) = from {
        from.1.metal_dirs.set_direction(to.0 - from.0, true);
        to.1.metal_dirs.set_direction(from.0 - to.0, true);
        (Some(from.1), to.1)
    } else {
        (None, to.1)
    }
}
