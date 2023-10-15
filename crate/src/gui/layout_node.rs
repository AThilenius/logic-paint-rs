use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    node::Node,
    types::{Alignment, Layout, Length, Point, Rect, Size},
};

// Wraps a node with a layout information, that is updated each layout cycle. This forms the layout
// tree.
pub struct LayoutNode<'a> {
    pub node: &'a dyn Node,
    pub layout: Layout,
    pub children: Vec<LayoutNode<'a>>,

    // The minimum size of the node, including margin, padding, border and content. This is ignored
    // completely for fixed size, though it's still computed for transitive layout.
    pub min_size: Size,

    // The outermost rect fully enclosing the node, including margin, padding and border.
    pub rect: Rect,
}

impl<'a> LayoutNode<'a> {
    pub fn new(node: &'a dyn Node) -> Self {
        Self {
            node,
            layout: node.layout(),
            children: node
                .children()
                .into_iter()
                .map(|child| LayoutNode::new(child))
                .collect(),
            min_size: Size::default(),
            rect: Rect::default(),
        }
    }

    pub fn layout_and_draw(&mut self, ctx: &CanvasRenderingContext2d) {
        self.compute_min_size();
        self.layout_children();
        self.draw(ctx);
    }

    // Depth-first traversal of the layout tree to compute the `min_size` of all nodes. This is
    // later used layout all nodes.
    fn compute_min_size(&mut self) {
        // Children are always computed first.
        for child in &mut self.children {
            child.compute_min_size();
        }

        // Our min_size is the sum of our children's min_size for the major axis, and the max of our
        // children's size for the minor axis.
        let children_min_size = if self.layout.alignment == Alignment::Column {
            Size {
                width: self
                    .children
                    .iter()
                    .map(|child| child.min_size.width)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0),
                height: self
                    .children
                    .iter()
                    .map(|child| child.min_size.height)
                    .sum::<f32>(),
            }
        } else {
            Size {
                width: self
                    .children
                    .iter()
                    .map(|child| child.min_size.width)
                    .sum::<f32>(),
                height: self
                    .children
                    .iter()
                    .map(|child| child.min_size.height)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0),
            }
        };

        // Allow the node itself to use that value to compute it's own min size.
        self.min_size = self.node.min_content_size(&self.layout, children_min_size);

        // Then finally add in margin, padding and border.
        self.min_size = self.min_size
            + self.layout.margin.sum()
            + self.layout.padding.sum()
            + self.layout.border.size.sum();
    }

    // Called AFTER compute_min_size, using the min_size to layout children in the correct position.
    fn layout_children(&mut self) {
        let column = self.layout.alignment == Alignment::Column;

        // Margin, padding and border are subtracted from the available working area.
        let working_area = self.rect.size
            - self.layout.margin.sum()
            - self.layout.padding.sum()
            - self.layout.border.size.sum();

        // We need to individually gather the min size for both fixed/auto children, and weighted
        // children. The min size for fixed/auto children is first subtracted from the working area,
        // then the remaining space is divided among the weighted children, but the min size for
        // each weighted child is also respected (eating into other weighted children's space or
        // overflowing if necessary). This is done in a single iteration over the children.

        let working_major_axis = if column {
            working_area.height
        } else {
            working_area.width
        };
        let mut fixed_children_min_size = 0.0;
        let mut weighted_children_total_weight = 0.0;

        for child in &self.children {
            if column {
                match child.layout.height {
                    Length::Pixels(_) | Length::Auto => {
                        fixed_children_min_size += child.min_size.height
                    }
                    Length::Weighted(weight) => {
                        weighted_children_total_weight += weight;
                    }
                }
            } else {
                match child.layout.width {
                    Length::Pixels(_) | Length::Auto => {
                        fixed_children_min_size += child.min_size.width
                    }
                    Length::Weighted(weight) => {
                        weighted_children_total_weight += weight;
                    }
                }
            }
        }

        // The working weight area is divided among weighted children. However, if a child's
        // min_size is larger than it's respective share, it's effectively 'booted' out of the
        // weighted pool and treated as a fixed/auto child. This requires a separate pass.
        let mut working_weighted_area = working_major_axis - fixed_children_min_size;

        for child in &mut self.children {
            let child_major_axis = if column {
                child.layout.height
            } else {
                child.layout.width
            };

            if let Length::Weighted(weight) = child_major_axis {
                let weight_ratio = weight / weighted_children_total_weight;
                let weighted_size = weight_ratio * working_weighted_area;
                let min_size = if column {
                    child.min_size.height
                } else {
                    child.min_size.width
                };

                if min_size > weighted_size {
                    // The child is effectively booted out of the weighted pool, and treated as a
                    // fixed/auto child.
                    if column {
                        child.layout.height = Length::Pixels(min_size);
                    } else {
                        child.layout.width = Length::Pixels(min_size);
                    }
                    weighted_children_total_weight -= weight;
                    working_weighted_area -= min_size;
                }
            }
        }

        let content_rect =
            self.rect - self.layout.margin - self.layout.padding - self.layout.border.size;
        let mut cursor = content_rect.origin;

        for child in &mut self.children {
            let child_major_axis = if column {
                child.layout.height
            } else {
                child.layout.width
            };

            let child_minor_axis = if column {
                child.layout.width
            } else {
                child.layout.height
            };

            let child_major_size = match child_major_axis {
                Length::Pixels(size) => size,
                Length::Auto => {
                    if column {
                        child.min_size.height
                    } else {
                        child.min_size.width
                    }
                }
                Length::Weighted(weight) => {
                    let weight_ratio = weight / weighted_children_total_weight;
                    let weighted_size = weight_ratio * working_weighted_area;
                    weighted_size
                }
            };

            let child_minor_size = match child_minor_axis {
                Length::Pixels(size) => size,
                Length::Auto => {
                    if column {
                        child.min_size.height
                    } else {
                        child.min_size.width
                    }
                }
                // Weighted minor axis is taken to mean: parent's size.
                Length::Weighted(_) => {
                    if column {
                        content_rect.size.width
                    } else {
                        content_rect.size.height
                    }
                }
            };

            child.rect = Rect {
                origin: cursor,
                size: Size {
                    height: if column {
                        child_major_size
                    } else {
                        child_minor_size
                    },
                    width: if column {
                        child_minor_size
                    } else {
                        child_major_size
                    },
                },
            };

            cursor = if column {
                cursor
                    + Point {
                        top: child_major_size,
                        left: 0.0,
                    }
            } else {
                cursor
                    + Point {
                        top: 0.0,
                        left: child_major_size,
                    }
            };

            // Let the child layout it's children (recursive decent).
            child.layout_children();
        }
    }

    fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
        // Draw the background and border first.
        ctx.set_stroke_style(
            &format!(
                "1px rgba({}, {}, {}, {})",
                self.layout.border.color.r,
                self.layout.border.color.g,
                self.layout.border.color.b,
                self.layout.border.color.a,
            )
            .into(),
        );
        ctx.stroke_rect(
            self.rect.origin.left as f64,
            self.rect.origin.top as f64,
            self.rect.size.width as f64,
            self.rect.size.height as f64,
        );

        if let Some(bg) = self.layout.background {
            ctx.set_fill_style(&format!("rgba({}, {}, {}, {})", bg.r, bg.g, bg.b, bg.a).into());
            ctx.fill_rect(
                self.rect.origin.left as f64,
                self.rect.origin.top as f64,
                self.rect.size.width as f64,
                self.rect.size.height as f64,
            );
        }

        self.node.draw(self.rect, ctx);

        // Then draw children
        for child in &mut self.children {
            child.draw(ctx);
        }
    }
}
