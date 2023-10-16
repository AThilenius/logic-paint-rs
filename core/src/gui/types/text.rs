use serde::{Deserialize, Serialize};
use web_sys::CanvasRenderingContext2d;

use crate::gui::types::{Color, Point, Size};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
    pub font_size: u32,
    pub color: Color,
    pub multiline: bool,
    pub line_spacing: f32,
}

impl Text {
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, point: Point) {
        ctx.set_font(&format!("{}px Courier New", self.font_size));
        ctx.set_fill_style(&self.color.to_canvas_string());
        let ascent = self.glyph_ascent() as f64;
        let spacing = self.font_size as f64 * self.line_spacing as f64;

        if self.multiline {
            for (i, line) in self.text.lines().enumerate() {
                // Canvas draws text from the baseline, so we have to offset an additional ascent.
                ctx.fill_text(
                    line,
                    point.left as f64,
                    point.top as f64 + ascent + (i as f64 * spacing),
                )
                .unwrap();
            }
        } else {
            ctx.fill_text(&self.text, point.left as f64, point.top as f64 + ascent)
                .unwrap();
        }
    }

    pub fn bounding_size(&self) -> Size {
        let advance = self.glyph_advance();
        let spacing = self.font_size as f32 * self.line_spacing;

        if self.multiline {
            let mut size = Size::default();

            for line in self.text.lines() {
                size.width = size.width.max(line.chars().count() as f32 * advance);
                size.height += spacing;
            }

            size
        } else {
            Size {
                width: self.text.chars().count() as f32 * advance,
                height: spacing,
            }
        }
    }

    pub fn glyph_ascent(&self) -> f32 {
        self.font_size as f32 * 0.8
    }

    pub fn glyph_advance(&self) -> f32 {
        self.font_size as f32 * 0.6
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16,
            color: Color::BLACK,
            multiline: true,
            line_spacing: 1.0,
        }
    }
}
