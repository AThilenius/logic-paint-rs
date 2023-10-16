use serde::{Deserialize, Serialize};
use web_sys::CanvasRenderingContext2d;

use crate::gui::types::{Background, Border, Color, Point, Rect, Text};

/// Wraps calls to a Canvas 2D context in a serializable type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderOp {
    Rect {
        rect: Rect,
        border: Option<Border>,
        background: Option<Background>,
    },
    Text {
        point: Point,
        text: Text,
    },
}

impl RenderOp {
    pub fn draw(self, ctx: &CanvasRenderingContext2d) {
        match self {
            RenderOp::Rect {
                rect,
                border,
                background,
            } => {
                if let Some(background) = background {
                    background.draw(rect, border, ctx);
                }

                if let Some(border) = border {
                    border.draw(rect, ctx);
                }
            }
            RenderOp::Text { point, text } => {
                text.draw(ctx, point);
            }
        }
    }
}
