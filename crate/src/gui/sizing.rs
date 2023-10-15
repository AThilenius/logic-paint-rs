use log::info;
use serde::{Deserialize, Serialize};
use web_sys::CanvasRenderingContext2d;

// Wraps a node with a layout information, that is updated each layout cycle. This forms the layout
// tree.
pub struct LayoutNode<'a> {
    pub node: &'a dyn Node,
    pub layout: Layout,
    pub children: Vec<LayoutNode<'a>>,

    // The minimum size of the node, including margin, padding, border and content. This is ignored
    // completely for fixed size, though it's still computed for transitive layout.
    pub min_size: Size2D,

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
            min_size: Size2D::default(),
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
            Size2D {
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
            Size2D {
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
        let mut weighted_children_min_size = 0.0;
        let mut weighted_children_total_weight = 0.0;

        for child in &self.children {
            if column {
                match child.layout.height {
                    Length::Pixels(_) | Length::Auto => {
                        fixed_children_min_size += child.min_size.height
                    }
                    Length::Weighted(weight) => {
                        weighted_children_min_size += child.min_size.height;
                        weighted_children_total_weight += weight;
                    }
                }
            } else {
                match child.layout.width {
                    Length::Pixels(_) | Length::Auto => {
                        fixed_children_min_size += child.min_size.width
                    }
                    Length::Weighted(weight) => {
                        weighted_children_min_size += child.min_size.width;
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
                size: Size2D {
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
                    + Point2D {
                        top: child_major_size,
                        left: 0.0,
                    }
            } else {
                cursor
                    + Point2D {
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
    fn min_content_size(&self, layout: &Layout, children_min_size: Size2D) -> Size2D {
        Size2D {
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Layout {
    pub alignment: Alignment,
    pub width: Length,
    pub height: Length,
    pub position: Position,
    pub margin: BoxSize,
    pub padding: BoxSize,
    pub border: Border,
    pub background: Option<Color>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Position {
    // The position is determined by the layout algorithm. This is analogous to flex-box layout in
    // CSS.
    Standard,

    // The position is fixed in viewport coordinates, starting a new, fresh layout tree. This is
    // much like fixed positioning in CSS.
    Fixed(Point2D),

    // The position is relative to where it would normally be placed. The element contributes to the
    // layout, but any gaps left are not filled.
    Relative(Point2D),
}

// Which axis is being dynamically layed out. The other axis is either treated as a fixed size or
// an auto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Alignment {
    // Children elements will be layed out horizontally, with the vertical axis will be computed
    // with the skyline algorithm.
    Column,

    // Children elements will be layed out vertically, with the horizontal axis will be computed
    // with the skyline algorithm.
    Row,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Length {
    // Size is determined by the sum of the desired sizes of its children.
    Auto,

    // Size will always be exactly this value, even if it results in overflow.
    Pixels(f32),

    // Size is determined by summing all weighted children, and dividing the non-fixed/auto
    // remaining space weighted among them. A single weighted child results in a "Full" size. A
    // weighted size used on a minor axis (ie. not the axis being layed out) is treated as "Auto".
    Weighted(f32),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Point2D {
    pub top: f32,
    pub left: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Size2D {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct BoxSize {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Rect {
    pub origin: Point2D,
    pub size: Size2D,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Border {
    pub size: BoxSize,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Position {
    fn default() -> Self {
        Self::Standard
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Column
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::Auto
    }
}

impl Layout {
    // The fixed size for each axis, otherwise zero.
    pub fn fixed_size(&self) -> Size2D {
        Size2D {
            width: self.width.min_size(),
            height: self.height.min_size(),
        }
    }

    // The weighted value for each axis, otherwise zero.
    pub fn weighted_value(&self) -> Size2D {
        Size2D {
            width: match self.width {
                Length::Weighted(weight) => weight,
                _ => 0.0,
            },
            height: match self.height {
                Length::Weighted(weight) => weight,
                _ => 0.0,
            },
        }
    }

    /// The offset from the origin of the layout node to the origin of the content. Aka origin + the
    /// upper-left of margin, padding and border.
    pub fn content_offset(&self) -> Point2D {
        self.margin.top_left() + self.padding.top_left() + self.border.size.top_left()
    }
}

impl Length {
    fn min_size(&self) -> f32 {
        match self {
            Self::Auto => 0.0,
            Self::Pixels(size) => *size,
            Self::Weighted(_) => 0.0,
        }
    }
}

impl Rect {
    pub fn bottom_right(&self) -> Point2D {
        Point2D {
            top: self.origin.top + self.size.height,
            left: self.origin.left + self.size.width,
        }
    }
}

#[allow(unused)]
impl BoxSize {
    pub const ZERO: BoxSize = BoxSize::equal(0.0);
    pub const ONE: BoxSize = BoxSize::equal(1.0);

    pub const fn equal(size: f32) -> Self {
        Self {
            left: size,
            top: size,
            right: size,
            bottom: size,
        }
    }

    pub fn sum(&self) -> Size2D {
        Size2D {
            width: self.left + self.right,
            height: self.top + self.bottom,
        }
    }

    pub fn top_left(&self) -> Point2D {
        Point2D {
            top: self.top,
            left: self.left,
        }
    }
}

#[allow(unused)]
impl Color {
    pub const AQUAMARINE: Self = Self::new(127, 255, 212, 255);
    pub const BEIGE: Self = Self::new(245, 245, 220, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const BROWN: Self = Self::new(165, 42, 42, 255);
    pub const CORAL: Self = Self::new(255, 127, 80, 255);
    pub const CYAN: Self = Self::new(0, 255, 255, 255);
    pub const DARK_GRAY: Self = Self::new(169, 169, 169, 255);
    pub const FOREST: Self = Self::new(34, 139, 34, 255);
    pub const GOLD: Self = Self::new(255, 215, 0, 255);
    pub const GRAY: Self = Self::new(128, 128, 128, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const INDIGO: Self = Self::new(75, 0, 130, 255);
    pub const KHAKI: Self = Self::new(240, 230, 140, 255);
    pub const LAVENDER: Self = Self::new(230, 230, 250, 255);
    pub const LIGHT_GRAY: Self = Self::new(211, 211, 211, 255);
    pub const LIME: Self = Self::new(0, 255, 0, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    pub const MAROON: Self = Self::new(128, 0, 0, 255);
    pub const MIDNIGHT: Self = Self::new(25, 25, 112, 255);
    pub const MINT: Self = Self::new(189, 252, 201, 255);
    pub const NAVY: Self = Self::new(0, 0, 128, 255);
    pub const OLIVE: Self = Self::new(128, 128, 0, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);
    pub const ORCHID: Self = Self::new(218, 112, 214, 255);
    pub const PEACH: Self = Self::new(255, 218, 185, 255);
    pub const PINK: Self = Self::new(255, 192, 203, 255);
    pub const PURPLE: Self = Self::new(128, 0, 128, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const SALMON: Self = Self::new(250, 128, 114, 255);
    pub const TEAL: Self = Self::new(0, 128, 128, 255);
    pub const TOMATO: Self = Self::new(255, 99, 71, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const TURQUOISE: Self = Self::new(64, 224, 208, 255);
    pub const WHEAT: Self = Self::new(245, 222, 179, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

// Sum for Size2D
impl std::iter::Sum for Size2D {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, size| acc + size)
    }
}

// Adding a Size2D to a Size2D
impl std::ops::Add<Size2D> for Size2D {
    type Output = Self;

    fn add(self, rhs: Size2D) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

// Subtracting BoxSize from Rect
impl std::ops::Sub<BoxSize> for Rect {
    type Output = Self;

    fn sub(self, rhs: BoxSize) -> Self::Output {
        Self {
            origin: Point2D {
                top: self.origin.top + rhs.top,
                left: self.origin.left + rhs.left,
            },
            size: Size2D {
                width: self.size.width - rhs.left - rhs.right,
                height: self.size.height - rhs.top - rhs.bottom,
            },
        }
    }
}

// Subtracting Size2D from Size2D
impl std::ops::Sub<Size2D> for Size2D {
    type Output = Self;

    fn sub(self, rhs: Size2D) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

// Add Point2D to Point2D
impl std::ops::Add<Point2D> for Point2D {
    type Output = Self;

    fn add(self, rhs: Point2D) -> Self::Output {
        Self {
            top: self.top + rhs.top,
            left: self.left + rhs.left,
        }
    }
}
