use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct ConstValue {
    pub bus_width: usize,
    pub value: i32,
}

impl ConstValue {
    pub fn new(bus_width: usize, value: i32) -> Self {
        Self { bus_width, value }
    }
}

impl Module for ConstValue {
    fn get_pins(&self) -> Vec<Pin> {
        let mut pins =
            Pin::new_repeating((0, 0).into(), (0, -1).into(), self.bus_width, "B", false);
        let unsigned = unsafe { std::mem::transmute::<i32, u32>(self.value) };

        for i in 0..self.bus_width {
            pins[i].output_high = (unsigned >> i) & 1 > 0;
        }

        pins
    }

    fn set_pins(&mut self, _pins: &Vec<Pin>) {
        // Output only, ignore.
    }
}

pub struct ConstValueComponent;

#[derive(Properties)]
pub struct ConstValueProps {
    pub data: Rc<RefCell<ConstValue>>,
}

impl PartialEq for ConstValueProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub enum Msg {}

impl Component for ConstValueComponent {
    type Message = Msg;
    type Properties = ConstValueProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data = ctx.props().data.borrow();

        html! {
            <div style={
                format!("
                        height: {}px;
                        width: {}px;
                        border: 1px solid darkgray;
                        text-align: center;
                        justify-content: middle;
                    ",
                    data.bus_width * 22,
                    22 * 1
                )
            }>
                <div style="position: absolute; transform: translate(0, -22px);">
                    {format!("{}", data.value)}
                </div>
            </div>
        }
    }
}
