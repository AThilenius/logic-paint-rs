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
    pub hovered: bool,
}

impl Widget for El {
    fn test_mouse_move(
        &mut self,
        layout: &mut Layout,
        children: &mut Vec<Node>,
        point: super::types::Point,
    ) {
        self.hovered = true;
    }

    fn draw(&self, render_queue: &mut Vec<RenderOp>, layout: &Layout, _children: &Vec<Node>) {
        render_queue.push(RenderOp::Rect {
            rect: layout.rect,
            border: self.border,
            background: Some(self.background),
        });

        if self.hovered {
            render_queue.push(RenderOp::Rect {
                rect: layout.rect,
                border: Some(Border {
                    size: super::types::BoxSize::Uniform(1.0),
                    color: Color::RED,
                    ..Default::default()
                }),
                background: None,
            });
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
