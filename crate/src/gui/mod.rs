mod sizing;

pub use sizing::*;
use web_sys::CanvasRenderingContext2d;

pub struct DemoUi {
    root: Element,
}

impl DemoUi {
    pub fn new() -> Self {
        Self {
            root: Element {
                layout: Layout {
                    background: Some(Color::RED),
                    alignment: Alignment::Row,
                    // Width and height are set each frame.
                    ..Default::default()
                },
                children: vec![
                    // Left panel
                    Box::new(Element {
                        layout: Layout {
                            background: Some(Color::GREEN),
                            width: Length::Pixels(100.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![
                            // First row of the left panel.
                            Box::new(Element {
                                layout: Layout {
                                    background: Some(Color::CYAN),
                                    alignment: Alignment::Row,
                                    width: Length::Weighted(1.0),
                                    ..Default::default()
                                },
                                // Three equally weighted elements in a row.
                                children: vec![
                                    Box::new(Element {
                                        layout: Layout {
                                            background: Some(Color::MAGENTA),
                                            width: Length::Weighted(1.0),
                                            height: Length::Pixels(20.0),
                                            ..Default::default()
                                        },
                                        children: vec![],
                                    }),
                                    Box::new(Element {
                                        layout: Layout {
                                            background: Some(Color::PINK),
                                            width: Length::Weighted(1.0),
                                            height: Length::Pixels(40.0),
                                            ..Default::default()
                                        },
                                        children: vec![],
                                    }),
                                    Box::new(Element {
                                        layout: Layout {
                                            background: Some(Color::GOLD),
                                            width: Length::Weighted(1.0),
                                            ..Default::default()
                                        },
                                        children: vec![
                                            // Three vertically stacked elements, each with a height
                                            // of 30 pixels.
                                            Box::new(Element {
                                                layout: Layout {
                                                    background: Some(Color::RED),
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                            }),
                                            Box::new(Element {
                                                layout: Layout {
                                                    background: Some(Color::GREEN),
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                            }),
                                            Box::new(Element {
                                                layout: Layout {
                                                    background: Some(Color::BLUE),
                                                    width: Length::Weighted(1.0),
                                                    height: Length::Pixels(30.0),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                            }),
                                        ],
                                    }),
                                ],
                            }),
                        ],
                    }),
                    // Center panel
                    Box::new(Element {
                        layout: Layout {
                            background: Some(Color::YELLOW),
                            width: Length::Weighted(1.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![],
                    }),
                    // Right panel
                    Box::new(Element {
                        layout: Layout {
                            background: Some(Color::ORANGE),
                            width: Length::Pixels(100.0),
                            height: Length::Weighted(1.0),
                            ..Default::default()
                        },
                        children: vec![],
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
            origin: Point2D {
                left: 0.0,
                top: 0.0,
            },
            size: Size2D { width, height },
        };
        layout.layout_and_draw(ctx);
    }
}

struct Element {
    layout: Layout,
    children: Vec<Box<dyn Node>>,
}

impl Node for Element {
    fn layout(&self) -> Layout {
        self.layout
    }

    fn children(&self) -> Vec<&dyn Node> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }
}
