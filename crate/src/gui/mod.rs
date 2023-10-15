mod layout_node;
pub mod node;
pub mod types;

pub use layout_node::*;
pub use node::*;
pub use types::*;

use web_sys::CanvasRenderingContext2d;

use node::El;

pub struct DemoUi {
    root: El,
}

impl DemoUi {
    pub fn new() -> Self {
        Self {
            root: El {
                layout: Layout {
                    alignment: Alignment::Row,
                    // Width and height are set each frame.
                    ..Default::default()
                },
                children: vec![
                    // Left panel
                    Box::new(StyledEl {
                        background: Background::Color(Color::GREEN),
                        layout: Layout {
                            width: Length::Pixels(100.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![
                            // First row of the left panel.
                            Box::new(StyledEl {
                                background: Background::Color(Color::CYAN),
                                layout: Layout {
                                    alignment: Alignment::Row,
                                    width: Length::Weighted(1.0),
                                    ..Default::default()
                                },
                                // Three equally weighted elements in a row.
                                children: vec![
                                    Box::new(StyledEl {
                                        background: Background::Color(Color::MAGENTA),
                                        layout: Layout {
                                            width: Length::Weighted(1.0),
                                            height: Length::Pixels(20.0),
                                            ..Default::default()
                                        },
                                        children: vec![],
                                        ..Default::default()
                                    }),
                                    Box::new(StyledEl {
                                        background: Background::Color(Color::PINK),
                                        layout: Layout {
                                            width: Length::Weighted(1.0),
                                            height: Length::Pixels(40.0),
                                            ..Default::default()
                                        },
                                        children: vec![],
                                        ..Default::default()
                                    }),
                                    Box::new(StyledEl {
                                        background: Background::Color(Color::GOLD),
                                        layout: Layout {
                                            width: Length::Weighted(1.0),
                                            ..Default::default()
                                        },
                                        children: vec![
                                            // Three vertically stacked elements, each with a height
                                            // of 30 pixels.
                                            Box::new(StyledEl {
                                                background: Background::Color(Color::RED),
                                                border: Some(Border {
                                                    size: BoxSize::Uniform(1.0),
                                                    color: Color::WHITE,
                                                    ..Default::default()
                                                }),
                                                layout: Layout {
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                                ..Default::default()
                                            }),
                                            Box::new(StyledEl {
                                                background: Background::Color(Color::GREEN),
                                                layout: Layout {
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                                ..Default::default()
                                            }),
                                            Box::new(StyledEl {
                                                background: Background::Color(Color::BLUE),
                                                border: Some(Border {
                                                    radius: BorderRadius::Uniform(20.0),
                                                    size: BoxSize::Uniform(2.0),
                                                    color: Color::WHITE,
                                                    ..Default::default()
                                                }),
                                                layout: Layout {
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                                ..Default::default()
                                            }),
                                        ],
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            }),
                        ],
                        ..Default::default()
                    }),
                    // Center panel
                    Box::new(StyledEl {
                        background: Background::Color(Color::YELLOW),
                        layout: Layout {
                            width: Length::Weighted(1.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![],
                        ..Default::default()
                    }),
                    // Right panel
                    Box::new(StyledEl {
                        background: Background::Color(Color::ORANGE),
                        layout: Layout {
                            width: Length::Pixels(100.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![],
                        ..Default::default()
                    }),
                ],
            },
        }
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
        // Get the size of the canvas.
        let width = ctx.canvas().unwrap().client_width() as f32;
        let height = ctx.canvas().unwrap().client_height() as f32;

        // Set the size of the root element.
        self.root.layout.width = Length::Pixels(width);
        self.root.layout.height = Length::Pixels(height);

        ctx.set_fill_style(&"lightgray".into());
        // Fill the background.
        ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

        // Layout and render the UI
        let mut layout = LayoutNode::new(&self.root);
        layout.rect = Rect {
            origin: Point {
                left: 0.0,
                top: 0.0,
            },
            size: Size { width, height },
        };
        layout.layout_and_draw(ctx);
    }
}
