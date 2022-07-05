use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use yew::html;

use crate::{
    coords::CellCoord,
    modules::{
        Alignment, Anchor, AnchoredModule, Memory, MemoryComponent, Module, Register,
        RegisterComponent, TogglePin, TogglePinComponent,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleSerde {
    pub anchor: AnchorSerde,
    pub module: ModuleSerdeData,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnchorSerde {
    pub root: CellCoord,
    pub align: Option<Alignment>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ModuleSerdeData {
    Memory(MemorySerdeData),
    Register(RegisterSerdeData),
    TogglePin(TogglePinSerdeData),
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

impl ModuleSerde {
    pub fn instantiate(&self) -> AnchoredModule {
        match self.module {
            ModuleSerdeData::Memory(MemorySerdeData {}) => {
                let module = Rc::new(RefCell::new(Memory::new()));

                AnchoredModule {
                    anchor: (&self.anchor).into(),
                    module: module.clone(),
                    html: html! { <MemoryComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::Register(RegisterSerdeData { bus_width }) => {
                let module = Rc::new(RefCell::new(Register::new(bus_width)));

                AnchoredModule {
                    anchor: (&self.anchor).into(),
                    module: module.clone(),
                    html: html! { <RegisterComponent data={module.clone()} /> },
                    module_serde: self.clone(),
                }
            }
            ModuleSerdeData::TogglePin(TogglePinSerdeData { initially_high }) => {
                let module = Rc::new(RefCell::new(TogglePin::new(
                    initially_high.unwrap_or_default(),
                )));

                AnchoredModule {
                    anchor: (&self.anchor).into(),
                    module: module.clone(),
                    html: html! { <TogglePinComponent data={module.clone()} /> },
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
        }
    }
}

impl From<&AnchorSerde> for Anchor {
    fn from(anchor_serde: &AnchorSerde) -> Self {
        Self {
            root: anchor_serde.root,
            align: anchor_serde.align.unwrap_or(Alignment::BottomLeft),
        }
    }
}
