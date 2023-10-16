use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    node::Node,
    types::{Background, Border, Color, Layout},
    widget::Widget,
};

/// A styled element, with a background and border. Useful for framing.
#[derive(Default)]
pub struct El {
    pub background: Background,
    pub border: Option<Border>,
    ctx: Option<CanvasRenderingContext2d>,
}

impl Widget for El {
    fn init(&mut self, ctx: CanvasRenderingContext2d) {
        self.ctx = Some(ctx);
    }

    fn draw(&self, layout: &Layout, _children: &Vec<Node>) {
        if let Some(ctx) = &self.ctx {
            self.background.draw(layout.rect, self.border, &ctx);

            if let Some(border) = &self.border {
                border.draw(layout.rect, &ctx);
            }
        }
    }
}

impl El {
    pub fn with_background_color(self, color: Color) -> Self {
        Self {
            background: Background::Color(color),
            ..self
        }
    }

    pub fn with_border(self, border: Border) -> Self {
        Self {
            border: Some(border),
            ..self
        }
    }
}
