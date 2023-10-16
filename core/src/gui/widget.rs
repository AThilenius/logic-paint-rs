use crate::gui::{
    node::Node,
    types::{Layout, Len, RenderOp, Size},
};

pub trait Widget {
    fn update(&mut self, layout: &mut Layout, children: &mut Vec<Node>) {
        let _ = layout;
        let _ = children;
    }

    fn draw(&self, render_queue: &mut Vec<RenderOp>, layout: &Layout, children: &Vec<Node>) {
        let _ = layout;
        let _ = children;
    }

    /// Compute the minimum size of this node's content, ignoring margin, padding and border. By
    /// default it returns the fixed-size of the node for fixed-size, and the sum of all children's
    /// min size for auto/weighted. This can be overridden to provide a custom implementation, for
    /// example basing the size on text layout.
    fn min_content_size(&self, layout: &Layout, children_min_size: Size) -> Size {
        Size {
            width: if let Len::Pixels(size) = layout.width {
                size
            } else {
                children_min_size.width
            },
            height: if let Len::Pixels(size) = layout.height {
                size
            } else {
                children_min_size.height
            },
        }
    }
}
