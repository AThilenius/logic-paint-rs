use bevy::math::IVec2;
use fast_hilbert::xy2h;

use crate::canvas::{Cell, DEFAULT_CANVAS_SIZE};

/// Component that represents and entire mutable Metal Oxide Silicon canvas.
pub struct CanvasData {
    pub size: u32,
    cells: Vec<Cell>,
}

impl Default for CanvasData {
    fn default() -> Self {
        Self {
            size: DEFAULT_CANVAS_SIZE as u32,
            cells: vec![Default::default(); DEFAULT_CANVAS_SIZE * DEFAULT_CANVAS_SIZE],
        }
    }
}

impl CanvasData {
    #[inline(always)]
    pub fn get_cell(&self, v: IVec2) -> &Cell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }

    #[inline(always)]
    pub fn get_cell_mut(&mut self, v: IVec2) -> &mut Cell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &mut self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }
}
