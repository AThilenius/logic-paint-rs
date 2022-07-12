use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use yew::html;

use crate::{
    coords::CellCoord,
    modules::{
        Clock, ClockComponent, ConstValue, ConstValueComponent, Memory, MemoryComponent, Module,
        Probe, ProbeComponent, Register, RegisterComponent, RootedModule, TogglePin,
        TogglePinComponent,
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
    Clock {},
    ConstValue {
        bus_width: usize,
        value: i32,
    },
    Probe {
        bus_width: usize,
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
            ModuleSerdeData::Clock {} => {
                let module = Rc::new(RefCell::new(Clock::new()));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ClockComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::ConstValue { bus_width, value } => {
                let module = Rc::new(RefCell::new(ConstValue::new(*bus_width, *value)));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ConstValueComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Probe { bus_width } => {
                let module = Rc::new(RefCell::new(Probe::new(*bus_width)));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ProbeComponent data={module.clone()} /> },
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
            ModuleSerdeData::Clock {} => Box::new(Clock::new()),
            ModuleSerdeData::ConstValue { bus_width, value } => {
                Box::new(ConstValue::new(*bus_width, *value))
            }
            ModuleSerdeData::Probe { bus_width } => Box::new(Probe::new(*bus_width)),
        }
    }
}
