pub use bundle::CanvasBundle;
pub use cell::*;
pub use data::*;
pub use plugin::*;

mod bundle;
mod cell;
mod data;
mod drawing;
mod input;
mod plugin;

pub const DEFAULT_CANVAS_SIZE: usize = 64;
