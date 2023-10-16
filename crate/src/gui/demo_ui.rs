use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    el::El,
    label::Label,
    node::Node,
    types::{Color, Len, Size, Text, ToPixels, ToWeighted},
};

pub struct DemoUi {
    root: Node,
}

impl DemoUi {
    pub fn new() -> Self {
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
                    .with_widget(Label {
                            text: Text {
                            text: "Hello, world!\nThis is a nice and long string to text the text sizing.\n|!@#$%^&*()\nAnd\nMore\nLines!\n\n\n- Foobar".to_string(),
                            ..Default::default()
                        }
                    })
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

        self.root.prepare(size);
        self.root.draw(&ctx);
    }
}
