use gloo::console::info;
use web_sys::CanvasRenderingContext2d;

use crate::gui::{
    el::El,
    node::Node,
    types::{Border, BoxSize, Color, Layout, Size, ToWeighted},
    widget::Widget,
};

/// A styled element, with a background and border. Useful for framing.
#[derive(Default)]
pub struct Label {
    pub font_size: u32,
    pub text: String,

    ctx: Option<CanvasRenderingContext2d>,
}

impl Widget for Label {
    fn init(&mut self, ctx: CanvasRenderingContext2d) {
        self.ctx = Some(ctx);
    }

    // fn update(&mut self, layout: &mut Layout, children: &mut Vec<Node>) {
    //     *children =
    //         vec![
    //             Node::row(1.weighted(), 1.weighted()).with_widget(El::default().with_border(
    //                 Border {
    //                     color: Color::BLACK,
    //                     size: BoxSize::Uniform(1.0),
    //                     ..Default::default()
    //                 },
    //             )),
    //         ]
    // }

    fn min_content_size(&self, _layout: &Layout, _children_min_size: Size) -> Size {
        if let Some(ctx) = &self.ctx {
            // Measure text.
            ctx.set_font(&format!("{}px Courier New", self.font_size));
            let text_metrics = ctx.measure_text(&self.text).unwrap();

            Size {
                width: text_metrics.width() as f32,
                height: (text_metrics.font_bounding_box_ascent()
                    + text_metrics.font_bounding_box_descent()) as f32,
            }
        } else {
            Size::ZERO
        }
    }

    fn draw(&self, layout: &Layout, children: &Vec<Node>) {
        if let Some(ctx) = &self.ctx {
            ctx.set_font(&format!("{}px Courier New", self.font_size));
            let text_metrics = ctx.measure_text(&self.text).unwrap();
            // Height without decent.
            let text_height = text_metrics.font_bounding_box_ascent();
            ctx.fill_text(
                &self.text,
                layout.rect.origin.left as f64,
                layout.rect.origin.top as f64 + text_height,
            )
            .unwrap();

            ctx.set_stroke_style(&"red".into());
            ctx.set_line_width(1.0);
            ctx.stroke_rect(
                layout.rect.origin.left as f64,
                layout.rect.origin.top as f64,
                layout.rect.size.width as f64,
                layout.rect.size.height as f64,
            );
        }
    }
}

impl Label {
    pub fn new(text: String) -> Self {
        Self {
            text,
            font_size: 16,
            ..Default::default()
        }
    }
}
