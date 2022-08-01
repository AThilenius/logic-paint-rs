use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Module, Pin},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Clock {
    pub root: CellCoord,
    pub delay: usize,
    pub devisor: usize,

    #[serde(skip)]
    high: bool,

    #[serde(skip)]
    pub edit_mode: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            high: false,
            delay: 1,
            devisor: 1,
            edit_mode: false,
        }
    }
}

impl Module for Clock {
    fn get_root(&self) -> CellCoord {
        self.root
    }
    fn set_edit_mode(&mut self, edit_mode: bool) {
        self.edit_mode = edit_mode;
    }

    fn get_pins(&self) -> Vec<Pin> {
        vec![Pin::new(0, 0, false, "CLK", false)]
    }

    fn clock(&mut self, _time: f64) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }

        self.delay = self.devisor;
        self.high = !self.high;
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
