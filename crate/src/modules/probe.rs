use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct Probe {
    pub bus_width: usize,
    pub value: i32,
}

impl Probe {
    pub fn new(bus_width: usize) -> Self {
        Self {
            bus_width,
            value: 0,
        }
    }
}

impl Module for Probe {
    fn get_pins(&self) -> Vec<Pin> {
        Pin::new_repeating((0, 0).into(), (0, -1).into(), self.bus_width, "B", false)
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        let mut unsigned = 0_u32;

        for i in 0..self.bus_width {
            if pins[i].input_high {
                unsigned |= 1 << i;
            }
        }

        self.value = unsafe { std::mem::transmute::<u32, i32>(unsigned) };
    }
}

pub struct ProbeComponent;

#[derive(Properties)]
pub struct ProbeProps {
    pub data: Rc<RefCell<Probe>>,
}

impl PartialEq for ProbeProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub enum Msg {}

impl Component for ProbeComponent {
    type Message = Msg;
    type Properties = ProbeProps;

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
                <div style="
                    position: absolute;
                    transform: translate(-2px, -26px);
                    padding: 3px;
                    background: #00c4ff;
                    font-weight: bold;
                    color: black;
                ">
                    {format!("{}", data.value)}
                </div>
            </div>
        }
    }
}
