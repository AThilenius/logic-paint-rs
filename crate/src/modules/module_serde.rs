use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use yew::html;

use crate::{
    coords::CellCoord,
    modules::{
        Clock, ClockComponent, Memory, MemoryComponent, Module, Register, RegisterComponent,
        RootedModule, TogglePin, TogglePinComponent,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleSerde {
    pub root: CellCoord,
    pub module: ModuleSerdeData,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ModuleSerdeData {
    Memory(MemorySerdeData),
    Register(RegisterSerdeData),
    TogglePin(TogglePinSerdeData),
    Clock(ClockSerdeData),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MemorySerdeData {}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisterSerdeData {
    pub bus_width: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TogglePinSerdeData {
    pub initially_high: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClockSerdeData {}

impl ModuleSerde {
    pub fn instantiate(&self) -> RootedModule {
        match self.module {
            ModuleSerdeData::Memory(MemorySerdeData {}) => {
                let module = Rc::new(RefCell::new(Memory::new()));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <MemoryComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Register(RegisterSerdeData { bus_width }) => {
                let module = Rc::new(RefCell::new(Register::new(bus_width)));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <RegisterComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::TogglePin(TogglePinSerdeData { initially_high }) => {
                let module = Rc::new(RefCell::new(TogglePin::new(
                    initially_high.unwrap_or_default(),
                )));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <TogglePinComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Clock(_) => {
                let module = Rc::new(RefCell::new(Clock::new()));

                RootedModule {
                    root: self.root,
                    module: module.clone(),
                    html: html! { <ClockComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
        }
    }
}

// We never go the other direction, so Into is implemented instead of From.
impl Into<Box<dyn Module>> for ModuleSerde {
    fn into(self) -> Box<dyn Module> {
        match self.module {
            ModuleSerdeData::Memory(MemorySerdeData {}) => Box::new(Memory::new()),
            ModuleSerdeData::Register(RegisterSerdeData { bus_width }) => {
                Box::new(Register::new(bus_width))
            }
            ModuleSerdeData::TogglePin(TogglePinSerdeData { initially_high }) => {
                Box::new(TogglePin::new(initially_high.unwrap_or_default()))
            }
            ModuleSerdeData::Clock(_) => Box::new(Clock::new()),
        }
    }
}
