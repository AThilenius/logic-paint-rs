use super::Anchor;

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::Pin;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TogglePinData {
    pub anchor: Anchor,
    pub pin: Pin,
}

impl TogglePinData {
    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn get_pins(&self) -> Vec<Pin> {
        vec![self.pin.clone()]
    }

    pub fn set_input_pins(&mut self, _states: &Vec<bool>) {
        // Toggle pins are exclusively driven, so we don't care what the ExecutionContext sets the
        // value to.
    }
}

pub struct TogglePinComponent;

#[derive(Properties)]
pub struct TogglePinProps {
    pub data: Rc<RefCell<TogglePinData>>,
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
