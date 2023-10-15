use serde::{Deserialize, Serialize};
use web_sys::CanvasRenderingContext2d;

use super::{BoxSize, Color, Rect};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Border {
    pub radius: BorderRadius,
    pub size: BoxSize,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum BorderRadius {
    None,
    Uniform(f32),
}

impl Border {
    pub fn draw(&self, rect: Rect, ctx: &CanvasRenderingContext2d) {
        ctx.set_stroke_style(
            &format!(
                "rgba({}, {}, {}, {})",
                self.color.r, self.color.g, self.color.b, self.color.a
            )
            .into(),
        );

        match self.size {
            BoxSize::Uniform(size) => {
                ctx.set_line_width(size as f64);

                // Only Uniform border sizes allow for rounded corners. Otherwise we ignore the
                // border radius.
                match self.radius {
                    BorderRadius::None => {
                        ctx.stroke_rect(
                            rect.origin.left as f64,
                            rect.origin.top as f64,
                            rect.size.width as f64,
                            rect.size.height as f64,
                        );
                    }
                    BorderRadius::Uniform(radius) => {
                        ctx.begin_path();
                        let _ = ctx.round_rect_with_f64(
                            rect.origin.left as f64,
                            rect.origin.top as f64,
                            rect.size.width as f64,
                            rect.size.height as f64,
                            radius as f64,
                        );
                        ctx.stroke();
                    }
                }
            }
            BoxSize::NonUniform {
                left,
                top,
                right,
                bottom,
            } => {
                // Border radius is ignored for non-uniform borders, because it makes very little
                // sense.
                ctx.begin_path();
                ctx.move_to(rect.origin.left as f64, rect.origin.top as f64);
                ctx.set_line_width(top as f64);
                ctx.line_to(
                    rect.origin.left as f64 + rect.size.width as f64,
                    rect.origin.top as f64,
                );
                ctx.set_line_width(right as f64);
                ctx.line_to(
                    rect.origin.left as f64 + rect.size.width as f64,
                    rect.origin.top as f64 + rect.size.height as f64,
                );
                ctx.set_line_width(bottom as f64);
                ctx.line_to(
                    rect.origin.left as f64,
                    rect.origin.top as f64 + rect.size.height as f64,
                );
                ctx.set_line_width(left as f64);
                ctx.line_to(rect.origin.left as f64, rect.origin.top as f64);
                ctx.stroke();
            }
        }
    }
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self::None
    }
}
