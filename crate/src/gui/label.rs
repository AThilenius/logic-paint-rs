use crate::gui::{
    node::Node,
    types::{Border, BoxSize, Color, Layout, RenderOp, Size, Text},
    widget::Widget,
};

/// A styled element, with a background and border. Useful for framing.
#[derive(Default)]
pub struct Label {
    pub text: Text,
}

impl Widget for Label {
    fn min_content_size(&self, _layout: &Layout, _children_min_size: Size) -> Size {
        self.text.bounding_size()
    }

    fn draw(&self, render_queue: &mut Vec<RenderOp>, layout: &Layout, _children: &Vec<Node>) {
        render_queue.push(RenderOp::Rect {
            rect: layout.rect,
            background: None,
            border: Some(Border {
                size: BoxSize::Uniform(1.0),
                color: Color::BLACK,
                ..Default::default()
            }),
        });

        render_queue.push(RenderOp::Text {
            text: self.text.clone(),
            point: layout.rect.origin,
        });
    }
}

impl Label {
    pub fn new(text: String) -> Self {
        Self {
            text: Text {
                text,
                ..Default::default()
            },
        }
    }
}
