use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    el::El,
    label::Label,
    node::Node,
    types::{Border, BoxSize, Color, Len, Size, ToPixels, ToWeighted},
};

pub struct DemoUi {
    root: Node,
}

impl DemoUi {
    pub fn new() -> Self {
        // Self {
        //     root: Node {
        //         layout: Layout {
        //             alignment: Alignment::Row,
        //             ..Default::default()
        //         },
        //         widget: Box::new(El {
        //             background: Background::Color(Color::RED),
        //             ..Default::default()
        //         }),
        //         children: vec![
        //             // Left panel
        //             Node {
        //                 layout: Layout {
        //                     width: Length::Pixels(200.0),
        //                     height: Length::Weighted(1.0),
        //                     ..Default::default()
        //                 },
        //                 widget: Box::new(El {
        //                     background: Background::Color(Color::GREEN),
        //                     ..Default::default()
        //                 }),
        //                 children: vec![],
        //             },
        //         ],
        //     },
        // }

        let widget_row = Node::row(1.weighted(), Len::Auto)
            .with_widget(El::default().with_background_color(Color::CYAN))
            .with_child(
                Node::row(1.weighted(), 20.pixels())
                    .with_widget(El::default().with_background_color(Color::MAGENTA)),
            )
            .with_child(
                Node::column(1.weighted(), Len::Auto)
                    .with_widget(El::default().with_background_color(Color::PINK))
                    .with_child(
                        Node::row(1.weighted(), 15.pixels())
                            .with_widget(El::default().with_background_color(Color::RED)),
                    )
                    .with_child(
                        Node::row(1.weighted(), 15.pixels())
                            .with_widget(El::default().with_background_color(Color::GREEN)),
                    )
                    .with_child(
                        Node::row(1.weighted(), 15.pixels())
                            .with_widget(El::default().with_background_color(Color::BLUE)),
                    ),
            )
            .with_child(
                Node::row(2.weighted(), 20.pixels())
                    .with_widget(El::default().with_background_color(Color::GOLD)),
            );

        let left_panel = Node::column(200.pixels(), 1.weighted())
            .with_widget(El::default().with_background_color(Color::GREEN))
            .with_child(widget_row)
            .with_child(Node::row(1.weighted(), 1.weighted()))
            .with_child(
                Node::row(1.weighted(), 200.pixels())
                    .with_widget(El::default().with_background_color(Color::FOREST)),
            );

        let center_panel = Node::column(1.weighted(), 1.weighted())
            .with_child(
                Node::row(1.weighted(), 100.pixels())
                    .with_widget(El::default().with_background_color(Color::RED)),
            )
            .with_child(
                Node::row(Len::Auto, Len::Auto)
                    .with_widget(Label::new("Hello, world!".to_string())),
            );

        let app = Node::row(1.weighted(), 1.weighted())
            .with_child(left_panel)
            .with_child(center_panel);

        Self {
            root: Node {
                children: vec![app],
                ..Default::default()
            },
        }
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
        // Get the size of the canvas.
        let size = Size {
            width: ctx.canvas().unwrap().client_width() as f32,
            height: ctx.canvas().unwrap().client_height() as f32,
        };

        self.root.prepare(size, ctx.clone());
        self.root.draw();
    }
}
