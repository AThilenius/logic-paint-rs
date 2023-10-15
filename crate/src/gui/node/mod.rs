use web_sys::CanvasRenderingContext2d;

use crate::gui::types::{Layout, Length, Rect, Size};

pub mod el;
pub mod styled_el;

pub use el::*;
pub use styled_el::*;

pub trait Node {
    fn layout(&self) -> Layout {
        Layout::default()
    }

    fn children(&self) -> Vec<&dyn Node> {
        Vec::new()
    }

    fn draw(&self, _rect: Rect, _ctx: &CanvasRenderingContext2d) {}

    /// Compute the minimum size of this node's content, ignoring margin, padding and border. By
    /// default it returns the fixed-size of the node for fixed-size, and the sum of all children's
    /// min size for auto/weighted. This can be overridden to provide a custom implementation, for
    /// example basing the size on text layout.
    fn min_content_size(&self, layout: &Layout, children_min_size: Size) -> Size {
        Size {
            width: if let Length::Pixels(size) = layout.width {
                size
            } else {
                children_min_size.width
            },
            height: if let Length::Pixels(size) = layout.height {
                size
            } else {
                children_min_size.height
            },
        }
    }
}
