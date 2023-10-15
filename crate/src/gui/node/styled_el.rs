use web_sys::CanvasRenderingContext2d;

use crate::gui::{node::Node, Background, Border, Layout, Rect};

/// A styled element, with a background and border. Useful for framing.
#[derive(Default)]
pub struct StyledEl {
    pub layout: Layout,
    pub children: Vec<Box<dyn Node>>,
    pub background: Background,
    pub border: Option<Border>,
}

impl Node for StyledEl {
    fn layout(&self) -> Layout {
        self.layout
    }

    fn children(&self) -> Vec<&dyn Node> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn draw(&self, rect: Rect, ctx: &CanvasRenderingContext2d) {
        self.background.draw(rect, self.border, ctx);

        if let Some(border) = &self.border {
            border.draw(rect, ctx);
        }
    }
}
