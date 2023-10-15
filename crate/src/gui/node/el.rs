use crate::gui::{node::Node, Layout};

/// A generic element, useful for layout.
#[derive(Default)]
pub struct El {
    pub layout: Layout,
    pub children: Vec<Box<dyn Node>>,
}

impl Node for El {
    fn layout(&self) -> Layout {
        self.layout
    }

    fn children(&self) -> Vec<&dyn Node> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }
}
