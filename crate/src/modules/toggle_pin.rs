use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct TogglePin {
    pub pin: Pin,
}

impl TogglePin {
    pub fn new(initially_high: bool) -> Self {
        Self {
            pin: Pin::new(0, 0, initially_high),
        }
    }
}

impl Module for TogglePin {
    fn get_pins(&self) -> Vec<Pin> {
        vec![self.pin]
    }
}

pub struct TogglePinComponent;

#[derive(Properties)]
pub struct TogglePinProps {
    pub data: Rc<RefCell<TogglePin>>,
}

impl PartialEq for TogglePinProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub enum Msg {
    Clicked,
}

impl Component for TogglePinComponent {
    type Message = Msg;
    type Properties = TogglePinProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Clicked => {
                let mut data = ctx.props().data.borrow_mut();
                data.pin.output_high = !data.pin.output_high;
                true
            }
        }
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
                onclick={ctx.link().callback(|_| Msg::Clicked)}
            />
        }
    }
}
