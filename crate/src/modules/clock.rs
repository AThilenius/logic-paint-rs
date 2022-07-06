use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct Clock {
    pub pin: Pin,
}

impl Module for Clock {
    fn get_pins(&self) -> Vec<Pin> {
        vec![self.pin]
    }

    fn tick(&mut self, _time: f64) {
        self.pin.output_high = !self.pin.output_high;
    }
}

pub struct ClockComponent;

#[derive(Properties)]
pub struct ClockProps {
    pub data: Rc<RefCell<Clock>>,
}

impl PartialEq for ClockProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Component for ClockComponent {
    type Message = ();
    type Properties = ClockProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data = ctx.props().data.borrow();
        html! {
            <div
                style={
                    format!("
                            margin-left: 4px;
                            margin-bottom: 4px;
                            height: 14px;
                            width: 14px;
                            background: {}
                        ",
                        if data.pin.output_high { "red" } else { "blue" }
                    )
                }
            />
        }
    }
}
