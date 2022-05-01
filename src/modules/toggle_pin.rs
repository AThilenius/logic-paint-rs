use super::Anchor;

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::coords::CellCoord;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TogglePinData {
    pub anchor: Anchor,
    pub active: bool,
}

impl TogglePinData {
    pub fn reset(&mut self) {}

    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn get_pins(&self) -> Vec<CellCoord> {
        vec![self.anchor.root]
    }
}

pub struct TogglePinComponent;

#[derive(Properties)]
pub struct TogglePinProps {
    pub data: Rc<RefCell<TogglePinData>>,
}

impl PartialEq for TogglePinProps {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub enum Msg {
    Clicked,
}

impl Component for TogglePinComponent {
    type Message = Msg;
    type Properties = TogglePinProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Clicked => {
                let mut pin = ctx.props().data.borrow_mut();
                pin.active = !pin.active;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let pin = ctx.props().data.borrow();
        html! {
            <div
                style={
                    format!("
                            height: 20px;
                            width: 20px;
                            background: {}
                        ",
                        if pin.active { "red" } else { "blue" }
                    )
                }
                onclick={ctx.link().callback(|_| Msg::Clicked)}
            />
        }
    }
}