use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    el::El,
    types::{Alignment, Layout, Len, Point, Rect, Size},
    widget::Widget,
};

pub struct Node {
    pub layout: Layout,
    pub children: Vec<Node>,
    pub widget: Box<dyn Widget>,
}

impl Node {
    pub fn row(width: Len, height: Len) -> Self {
        Self {
            layout: Layout {
                alignment: Alignment::Row,
                width,
                height,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn column(width: Len, height: Len) -> Self {
        Self {
            layout: Layout {
                alignment: Alignment::Column,
                width,
                height,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn with_layout(self, layout: Layout) -> Self {
        Self { layout, ..self }
    }

    pub fn with_widget<T>(self, widget: T) -> Self
    where
        T: Widget + 'static,
    {
        Self {
            widget: Box::new(widget),
            ..self
        }
    }

    // Adds one child to the node.
    pub fn with_child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub fn prepare(&mut self, size: Size) {
        self.layout.width = Len::Pixels(size.width);
        self.layout.height = Len::Pixels(size.height);

        self.compute_min_size();

        // Reset our size to the desired root size.
        self.layout.rect = Rect {
            origin: Point {
                left: 0.0,
                top: 0.0,
            },
            size,
        };

        self.layout_children();

        self.update();
    }

    fn update(&mut self) {
        self.widget.update(&mut self.layout, &mut self.children);

        for child in &mut self.children {
            child.update();
        }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        let mut render_queue = Vec::new();
        self.dispatch_draw_recursive(&mut render_queue);
        for render_op in render_queue.drain(..) {
            render_op.draw(ctx);
        }
    }

    fn dispatch_draw_recursive(&self, render_queue: &mut Vec<crate::gui::types::RenderOp>) {
        self.widget.draw(render_queue, &self.layout, &self.children);

        for child in &self.children {
            child.dispatch_draw_recursive(render_queue);
        }
    }

    // Depth-first traversal of the layout tree to compute the `layout.rect.size` of all nodes. This
    // is later used layout all nodes.
    fn compute_min_size(&mut self) {
        // Children are always computed first.
        for child in &mut self.children {
            child.compute_min_size();
        }

        // Our layout.rect.size is the sum of our children's layout.rect.size for the major axis,
        // and the max of our children's size for the minor axis.
        let children_min_size = if self.layout.alignment == Alignment::Column {
            Size {
                width: self
                    .children
                    .iter()
                    .map(|child| child.layout.rect.size.width)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0),
                height: self
                    .children
                    .iter()
                    .map(|child| child.layout.rect.size.height)
                    .sum::<f32>(),
            }
        } else {
            Size {
                width: self
                    .children
                    .iter()
                    .map(|child| child.layout.rect.size.width)
                    .sum::<f32>(),
                height: self
                    .children
                    .iter()
                    .map(|child| child.layout.rect.size.height)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0),
            }
        };

        // Allow the widget to use that value to compute it's own min size.
        self.layout.rect.size = self
            .widget
            .min_content_size(&self.layout, children_min_size);

        // Then finally add in margin and padding.
        self.layout.rect.size =
            self.layout.rect.size + self.layout.margin.sum() + self.layout.padding.sum();
    }

    // Called AFTER compute_min_size, using the layout.rect.size to layout children in the correct position.
    fn layout_children(&mut self) {
        let column = self.layout.alignment == Alignment::Column;

        // Margin and padding are subtracted from the available working area.
        let working_area =
            self.layout.rect.size - self.layout.margin.sum() - self.layout.padding.sum();

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
                    Len::Pixels(_) | Len::Auto => {
                        fixed_children_min_size += child.layout.rect.size.height
                    }
                    Len::Weighted(weight) => {
                        weighted_children_total_weight += weight;
                    }
                }
            } else {
                match child.layout.width {
                    Len::Pixels(_) | Len::Auto => {
                        fixed_children_min_size += child.layout.rect.size.width
                    }
                    Len::Weighted(weight) => {
                        weighted_children_total_weight += weight;
                    }
                }
            }
        }

        // The working weight area is divided among weighted children. However, if a child's
        // layout.rect.size is larger than it's respective share, it's effectively 'booted' out of
        // the weighted pool and treated as a fixed/auto child. This requires a separate pass.
        let mut working_weighted_area = working_major_axis - fixed_children_min_size;

        for child in &mut self.children {
            let child_major_axis = if column {
                child.layout.height
            } else {
                child.layout.width
            };

            if let Len::Weighted(weight) = child_major_axis {
                let weight_ratio = weight / weighted_children_total_weight;
                let weighted_size = weight_ratio * working_weighted_area;
                let min_size = if column {
                    child.layout.rect.size.height
                } else {
                    child.layout.rect.size.width
                };

                if min_size > weighted_size {
                    // The child is effectively booted out of the weighted pool, and treated as a
                    // fixed/auto child.
                    if column {
                        child.layout.height = Len::Pixels(min_size);
                    } else {
                        child.layout.width = Len::Pixels(min_size);
                    }
                    weighted_children_total_weight -= weight;
                    working_weighted_area -= min_size;
                }
            }
        }

        let content_rect = self.layout.rect - self.layout.margin - self.layout.padding;
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
                Len::Pixels(size) => size,
                Len::Auto => {
                    if column {
                        child.layout.rect.size.height
                    } else {
                        child.layout.rect.size.width
                    }
                }
                Len::Weighted(weight) => {
                    let weight_ratio = weight / weighted_children_total_weight;
                    let weighted_size = weight_ratio * working_weighted_area;
                    weighted_size
                }
            };

            let child_minor_size = match child_minor_axis {
                Len::Pixels(size) => size,
                Len::Auto => {
                    if column {
                        child.layout.rect.size.width
                    } else {
                        child.layout.rect.size.height
                    }
                }
                // Weighted minor axis is taken to mean: parent's size.
                Len::Weighted(_) => {
                    if column {
                        content_rect.size.width
                    } else {
                        content_rect.size.height
                    }
                }
            };

            child.layout.rect = Rect {
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
}

impl Default for Node {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            children: vec![],
            widget: Box::new(El::default()),
        }
    }
}
