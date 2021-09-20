pub use bundle::{CanvasRenderBundle, CELL_WORLD_SIZE};
pub use plugin::{CanvasRenderPlugin, CellMaterial};

pub const CANVAS_DEPTH: f32 = 100.0;
pub const LABEL_DEPTH: f32 = 200.0;

mod bundle;
mod plugin;
mod rasterizer;
