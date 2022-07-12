use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct TogglePin {
    pub label: Option<String>,
    pub pin: Pin,
}

impl TogglePin {
    pub fn new(label: &Option<String>, initially_high: bool) -> Self {
        Self {
            label: label.clone(),
            pin: Pin::new(0, 0, initially_high, "CONST", false),
        }
    }
}

impl Module for TogglePin {
    fn get_pins(&self) -> Vec<Pin> {
        let mut pin = self.pin.clone();

        pin.label = if let Some(label) = &self.label {
            label.to_owned()
        } else {
            if self.pin.output_high {
                format!("HIGH")
            } else {
                format!("LOW")
            }
        };

        vec![pin]
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
        html! {
            <div
                style={"height: 20px; width: 20px;"}
                onclick={ctx.link().callback(|_| Msg::Clicked)}
            />
        }
    }
}
