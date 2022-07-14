use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

pub struct Clock {
    pub pin: Pin,
    delay: usize,
    devisor: usize,
}

impl Clock {
    pub fn new(start_delay: usize, devisor: usize) -> Self {
        Self {
            pin: Pin::new(0, 0, false, "CLK", false),
            delay: start_delay,
            devisor,
        }
    }
}

impl Module for Clock {
    fn get_pins(&self) -> Vec<Pin> {
        vec![self.pin.clone()]
    }

    fn clock(&mut self, _time: f64) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }

        self.delay = self.devisor;
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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // let data = ctx.props().data.borrow();
        html! {}
    }
}
