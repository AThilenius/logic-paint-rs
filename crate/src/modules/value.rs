use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Module, Pin},
    utils::{
        cell_offset::CellOffset, input::TextInput, local_cell_offset::LocalCellOffset,
        standard_pin::StandardPin,
    },
    wgl2::Camera,
};

use super::ConcreteModule;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Value {
    pub root: CellCoord,
    pub bus_width: usize,
    pub value: i64,
    pub spacing: usize,

    #[serde(skip)]
    pub value_in: i64,
}

impl Default for Value {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            bus_width: 1,
            value: 0,
            value_in: 0,
            spacing: 1,
        }
    }
}

impl Module for Value {
    fn get_root(&self) -> CellCoord {
        return self.root;
    }

    fn set_root(&mut self, root: CellCoord) {
        self.root = root;
    }

    fn get_pins(&self) -> Vec<Pin> {
        let mut pins = Pin::new_repeating(
            (0, 0).into(),
            (0, -(self.spacing as i32)).into(),
            self.bus_width,
            "b",
            false,
        );

        let unsigned = unsafe { std::mem::transmute::<i64, u64>(self.value) };
        for i in 0..self.bus_width {
            pins[i].output_high = (unsigned >> i) & 1 > 0;
        }

        pins
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        let mut unsigned = 0_u64;

        for i in 0..self.bus_width {
            if pins[i].input_high {
                unsigned |= 1 << i;
            }
        }

        self.value_in = unsafe { std::mem::transmute::<u64, i64>(unsigned) };
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub module: Value,
    pub camera: Camera,
    pub update_self: Callback<(bool, CellCoord, Option<ConcreteModule>)>,
    pub edit_mode: bool,
}

#[function_component(ValueComponent)]
pub fn value_component(props: &Props) -> Html {
    let Props {
        module,
        camera,
        update_self,
        edit_mode,
    } = props;

    let show_settings = use_state(|| false);

    let bus_width_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |e: String| {
            update_self.emit((
                true,
                module.get_root(),
                Some(ConcreteModule::Value(Value {
                    bus_width: e.parse::<usize>().unwrap_or(1).min(64),
                    ..(module)
                })),
            ));
        })
    };

    let spacing_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |e: String| {
            update_self.emit((
                true,
                module.get_root(),
                Some(ConcreteModule::Value(Value {
                    spacing: e.parse::<usize>().unwrap_or(1).max(1).min(100),
                    ..(module)
                })),
            ));
        })
    };

    let delete_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |_| {
            update_self.emit((true, module.get_root(), None));
        })
    };

    html! {
        <CellOffset camera={camera.clone()} root={module.root} >
            <LocalCellOffset amount={IVec2::new(0, 1)}>
                <div class={classes!("lp-cell-center")}>
                    {format!("{}", module.value | module.value_in)}
                </div>
            </LocalCellOffset>
            <StandardPin pins={module.get_pins()} on_click={
                let module = module.clone();
                let update_self = update_self.clone();
                Callback::from(move |i| {
                    update_self.emit((
                        false,
                        module.get_root(),
                        Some(ConcreteModule::Value(Value {
                        value: module.value ^ (1_i64 << i),
                        ..(module)
                    }))));
                })
            } />
            {
                if *edit_mode {
                    html! {
                        <div
                            class={classes!("lp-module-edit-mode-div", "lp-pointer-events")}
                            onclick={
                                let show_settings = show_settings.clone();
                                Callback::from(move |_| show_settings.set(!*show_settings))
                            }
                        >
                            {"⚙"}
                        </div>
                    }
                } else {
                    html!()
                }
            }
            {
                if *edit_mode && *show_settings {
                    html! {
                        <LocalCellOffset amount={IVec2::new(1, 0)}>
                            <div class={classes!("lp-settings-panel", "lp-pointer-events")}>
                                <div style="
                                    background: red;
                                    margin-bottom: 4px;
                                    padding: 0 2px;"
                                    onclick={delete_on_change}>
                                    {"DEL"}
                                </div>
                                <TextInput
                                    label="Bus Width"
                                    on_change={bus_width_on_change}
                                    value={format!("{}", module.bus_width)}
                                    width={24.0}
                                />
                                <TextInput
                                    label="Spacing"
                                    on_change={spacing_on_change}
                                    value={format!("{}", module.spacing)}
                                    width={24.0}
                                />
                            </div>
                        </LocalCellOffset>
                    }
                } else {
                    html!()
                }
            }
        </CellOffset>
    }
}
