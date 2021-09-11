pub use bundle::CanvasBundle;
pub use canvas::*;
pub use cell::*;
pub use plugin::*;

mod bundle;
mod canvas;
mod cell;
mod drawing;
mod graph;
mod input;
mod plugin;

pub const DEFAULT_CANVAS_SIZE: usize = 64;
