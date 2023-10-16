use crate::gui::{
    node::Node,
    types::{Background, Border, Color, Layout, RenderOp},
    widget::Widget,
};

/// A styled element, with a background and border. Useful for framing.
#[derive(Default)]
pub struct El {
    pub background: Background,
    pub border: Option<Border>,
}

impl Widget for El {
    fn draw(&self, render_queue: &mut Vec<RenderOp>, layout: &Layout, _children: &Vec<Node>) {
        render_queue.push(RenderOp::Rect {
            rect: layout.rect,
            border: self.border,
            background: Some(self.background),
        });
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
