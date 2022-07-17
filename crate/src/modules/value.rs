use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::{
    log,
    modules::{Module, Pin},
    utils::input::TextInput,
};

pub struct Value {
    pub(self) bus_width: usize,
    pub(self) value_out: i32,
    pub(self) value_in: i32,
    pub(self) spacing: usize,
    pub(self) out_en: bool,
}

impl Value {
    pub fn new(bus_width: usize, value_out: i32, spacing: usize, out_en: bool) -> Self {
        Self {
            bus_width,
            value_out,
            value_in: 0,
            spacing,
            out_en,
        }
    }
}

impl Module for Value {
    fn get_pins(&self) -> Vec<Pin> {
        let mut pins = Pin::new_repeating(
            (0, 0).into(),
            (0, -(self.spacing as i32)).into(),
            self.bus_width,
            "b",
            false,
        );

        if self.out_en {
            let unsigned = unsafe { std::mem::transmute::<i32, u32>(self.value_out) };
            for i in 0..self.bus_width {
                pins[i].output_high = (unsigned >> i) & 1 > 0;
            }
        }

        pins
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        let mut unsigned = 0_u32;

        for i in 0..self.bus_width {
            if pins[i].input_high {
                unsigned |= 1 << i;
            }
        }

        self.value_in = unsafe { std::mem::transmute::<u32, i32>(unsigned) };
    }
}

pub struct ConstValueComponent;

#[derive(Properties)]
pub struct ConstValueProps {
    pub data: Rc<RefCell<Value>>,
}

impl PartialEq for ConstValueProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub enum Msg {
    SetOutputValue(String),
    ToggleOutEn,
    ToggleOutBit(usize),
}

impl Component for ConstValueComponent {
    type Message = Msg;
    type Properties = ConstValueProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut data = ctx.props().data.borrow_mut();

        match msg {
            Msg::SetOutputValue(val) => {
                if let Ok(i) = val.parse::<i32>() {
                    log!("Setting value to {}", i);
                    data.value_out = i;
                } else {
                    data.value_out = 0;
                }
            }
            Msg::ToggleOutEn => data.out_en = !data.out_en,
            Msg::ToggleOutBit(i) => {
                data.value_out = data.value_out ^ (1 << i);
            }
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data = ctx.props().data.borrow();
        let on_change = ctx.link().callback(Msg::SetOutputValue);

        html! {
            <div style={
                format!("
                        height: {}px;
                        width: {}px;
                        border: 1px solid darkgray;
                        text-align: center;
                        justify-content: middle;
                    ",
                    data.bus_width * data.spacing * 22,
                    22 * 1
                )
            }>
                <div style="
                    position: absolute;
                    transform: translate(-10px, -55px);
                    padding: 3px;
                    background: #5e03fc;
                    color: white;
                    display: flex;
                    flex-direction: column;
                    width: 35px;
                ">
                    <TextInput
                        disabled={!data.out_en}
                        {on_change}
                        value={format!("{}", data.value_out)}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::ToggleOutEn)}>{"EN"}</button>
                </div>
                {
                    (0..data.bus_width).map(|i| html!{
                        <div
                            style="width: 22px; height: 22px;"
                            onclick={ctx.link().callback(move |_| Msg::ToggleOutBit(i))}
                        />
                    }).collect::<Html>()
                }
                <div style="
                    position: absolute;
                    padding: 3px;
                    transform: translate(-10px, 4px);
                    background: #00c4ff;
                    width: 35px;
                    text-align: center;
                    color: black;
                ">
                    {format!("{}", data.value_in)}
                </div>
            </div>
        }
    }
}
