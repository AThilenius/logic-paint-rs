use bevy::math::IVec2;
use fast_hilbert::xy2h;

use crate::canvas::DEFAULT_CANVAS_SIZE;

/// Component that represents and entire mutable Metal Oxide Silicon canvas.
pub struct CanvasData {
    pub size: u32,
    cells: Vec<CanvasCell>,
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
    pub fn get_cell(&self, v: IVec2) -> &CanvasCell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }

    pub fn get_cell_mut(&mut self, v: IVec2) -> &mut CanvasCell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &mut self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SiLayer {
    None,
    P,
    N,
    NOnP,
    POnN,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetalLayer {
    None,
    Metal,
    MetalAndVia,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayerState {
    Off,
    On,
    Err,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanvasCell {
    pub si: SiLayer,
    pub metal: MetalLayer,
    pub si_lower_state: LayerState,
    pub si_upper_state: LayerState,
    pub metal_state: LayerState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanvasCellChange {
    pub hilbert_code: u32,
    pub from_cell: CanvasCell,
    pub to_cell: CanvasCell,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            si: SiLayer::None,
            metal: MetalLayer::None,
            si_lower_state: LayerState::Off,
            si_upper_state: LayerState::Off,
            metal_state: LayerState::Off,
        }
    }
}
