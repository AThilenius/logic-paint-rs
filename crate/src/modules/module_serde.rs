use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use yew::html;

use crate::{
    coords::CellCoord,
    modules::{
        Clock, ClockComponent, ConstValueComponent, Memory, MemoryComponent, Module, Register,
        RegisterComponent, RootedModule, TogglePin, TogglePinComponent, Value,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleSerde {
    pub root: CellCoord,
    pub module: ModuleSerdeData,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ModuleSerdeData {
    Memory {},
    Register {
        bus_width: usize,
    },
    TogglePin {
        label: Option<String>,
        initially_high: Option<bool>,
    },
    Clock {
        start_delay: Option<usize>,
        devisor: Option<usize>,
    },
    Value {
        bus_width: usize,
        value: Option<i32>,
        spacing: Option<usize>,
        out_en: Option<bool>,
    },
}

impl ModuleSerde {
    pub fn instantiate(&self) -> RootedModule {
        match &self.module {
            ModuleSerdeData::Memory {} => {
                let module = Rc::new(RefCell::new(Memory::new()));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <MemoryComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Register { bus_width } => {
                let module = Rc::new(RefCell::new(Register::new(*bus_width)));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <RegisterComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::TogglePin {
                label,
                initially_high,
            } => {
                let module = Rc::new(RefCell::new(TogglePin::new(
                    label,
                    initially_high.unwrap_or_default(),
                )));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <TogglePinComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Clock {
                start_delay,
                devisor,
            } => {
                let module = Rc::new(RefCell::new(Clock::new(
                    start_delay.unwrap_or(4),
                    devisor.unwrap_or(4),
                )));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ClockComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Value {
                bus_width,
                value,
                spacing,
                out_en,
            } => {
                let module = Rc::new(RefCell::new(Value::new(
                    *bus_width,
                    value.unwrap_or(0),
                    spacing.unwrap_or(1),
                    out_en.unwrap_or(false),
                )));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ConstValueComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
        }
    }
}

// We never go the other direction, so Into is implemented instead of From.
impl Into<Box<dyn Module>> for ModuleSerde {
    fn into(self) -> Box<dyn Module> {
        match &self.module {
            ModuleSerdeData::Memory {} => Box::new(Memory::new()),
            ModuleSerdeData::Register { bus_width } => Box::new(Register::new(*bus_width)),
            ModuleSerdeData::TogglePin {
                label,
                initially_high,
            } => Box::new(TogglePin::new(label, initially_high.unwrap_or_default())),
            ModuleSerdeData::Clock {
                start_delay,
                devisor,
            } => Box::new(Clock::new(start_delay.unwrap_or(4), devisor.unwrap_or(4))),
            ModuleSerdeData::Value {
                bus_width,
                value,
                spacing,
                out_en,
            } => Box::new(Value::new(
                *bus_width,
                value.unwrap_or(0),
                spacing.unwrap_or(1),
                out_en.unwrap_or(false),
            )),
        }
    }
}
