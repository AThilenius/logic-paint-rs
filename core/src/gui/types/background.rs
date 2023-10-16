use serde::{Deserialize, Serialize};
use web_sys::CanvasRenderingContext2d;

use crate::gui::types::{Border, BorderRadius, Color, Rect};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Background {
    None,
    Color(Color),
}

impl Background {
    pub fn draw(&self, rect: Rect, border: Option<Border>, ctx: &CanvasRenderingContext2d) {
        match self {
            Self::None => {}
            Self::Color(color) => {
                ctx.set_fill_style(
                    &format!("rgba({}, {}, {}, {})", color.r, color.g, color.b, color.a).into(),
                );

                match border {
                    Some(Border {
                        radius: BorderRadius::Uniform(radius),
                        ..
                    }) => {
                        // Render the background with the border radius in mind.
                        ctx.begin_path();
                        let _ = ctx.round_rect_with_f64(
                            rect.origin.left as f64,
                            rect.origin.top as f64,
                            rect.size.width as f64,
                            rect.size.height as f64,
                            radius as f64,
                        );
                        ctx.fill();
                    }
                    Some(Border {
                        radius: BorderRadius::None,
                        ..
                    })
                    | None => {
                        // Render the background without the border radius.
                        ctx.fill_rect(
                            rect.origin.left as f64,
                            rect.origin.top as f64,
                            rect.size.width as f64,
                            rect.size.height as f64,
                        );
                    }
                }
            }
        }
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::None
    }
}
