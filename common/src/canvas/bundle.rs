use bevy::prelude::*;

use super::{CanvasData, drawing::CanvasDrawing, input::CanvasInput};

#[derive(Bundle, Default)]
pub struct CanvasBundle {
    canvas_data: CanvasData,
    canvas_input: CanvasInput,
    canvas_drawing: CanvasDrawing,
}
