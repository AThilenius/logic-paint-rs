use bevy::math::IVec2;
use fast_hilbert::xy2h;

pub const DEFAULT_CANVAS_SIZE: usize = 64;

/// Component that represents and entire mutable Metal Oxide Silicon canvas.
pub struct Canvas {
    pub size: u32,
    cells: Vec<CanvasCell>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            size: DEFAULT_CANVAS_SIZE as u32,
            cells: vec![Default::default(); DEFAULT_CANVAS_SIZE * DEFAULT_CANVAS_SIZE],
        }
    }
}

impl Canvas {
    pub fn get_cell(&self, v: IVec2) -> &CanvasCell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }

    pub fn get_cell_mut(&mut self, v: IVec2) -> &mut CanvasCell {
        assert!(v.cmpge(IVec2::ZERO).all());
        &mut self.cells[xy2h(v.x as u32, v.y as u32) as usize]
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SiLayerType {
    None,
    PType,
    NType,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LayerState {
    Inactive,
    Active,
    Error,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SiLayer {
    pub layer_type: SiLayerType,
    pub state: LayerState,
}

impl Default for SiLayer {
    fn default() -> Self {
        Self {
            layer_type: SiLayerType::None,
            state: LayerState::Inactive,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CanvasCell {
    pub lower_si_layer: SiLayer,
    pub upper_si_layer: SiLayer,
    pub has_metal: bool,
    pub has_via: bool,
    pub metal_state: LayerState,
}

impl Default for CanvasCell {
    fn default() -> Self {
        Self {
            lower_si_layer: SiLayer::default(),
            upper_si_layer: SiLayer::default(),
            has_metal: false,
            has_via: false,
            metal_state: LayerState::Inactive,
        }
    }
}
