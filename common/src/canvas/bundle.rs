use bevy::prelude::*;

use super::{drawing::CanvasDrawing, input::CanvasInput, Canvas};

#[derive(Bundle, Default)]
pub struct CanvasBundle {
    canvas: Canvas,
    canvas_input: CanvasInput,
    canvas_drawing: CanvasDrawing,
}
