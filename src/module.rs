use std::{cell::RefCell, rc::Rc};

use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::coords::CellCoord;

pub struct ModuleMount {
    pub root: CellCoord,
    pub align: ModuleAlignment,
    pub module: Option<Rc<RefCell<Module>>>,
}

#[derive(Serialize, Deserialize)]
pub enum Module {
    One { string: String },
    Two { number: i32, vec: Vec<u8> },
}
#[derive(Properties, PartialEq)]
pub struct ModuleProps {
    #[prop_or(CellCoord(IVec2::ZERO))]
    pub root: CellCoord,

    #[prop_or(ModuleAlignment::UpperLeft)]
    pub align: ModuleAlignment,

    #[prop_or(None)]
    pub module: Option<Rc<RefCell<Module>>>,
}

pub enum ModuleMsg {
    Update,
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleAlignment {
    UpperLeft,
    UpperRight,
}

impl From<(CellCoord, ModuleAlignment, Module)> for ModuleMount {
    fn from((root, align, module): (CellCoord, ModuleAlignment, Module)) -> Self {
        Self {
            root,
            align,
            module: Some(Rc::new(RefCell::new(module))),
        }
    }
}

impl PartialEq for Module {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Component for ModuleMount {
    type Message = ModuleMsg;
    type Properties = ModuleProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            root: ctx.props().root,
            align: ctx.props().align,
            module: ctx.props().module.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ModuleMsg::Update => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if ctx.props().module.is_none() {
            return html!();
        }

        let html = match &*ctx.props().module.as_ref().unwrap().borrow() {
            Module::One { string } => {
                html!(<div style="pointer-events:auto;width:200px;height:100px;background:red;">{format!("One: {}", string)}</div>)
            }
            Module::Two { number, vec } => {
                html!(<div style="width:100px;height:50px;background:blue;">{format!("Two: {}, {:#?}", number, vec)}</div>)
            }
        };

        html
    }
}
