use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{ConcreteModule, Module, Pin},
    utils::{
        cell_offset::CellOffset, input::TextInput, local_cell_offset::LocalCellOffset,
        standard_pin::StandardPin,
    },
    wgl2::Camera,
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Clock {
    pub root: CellCoord,
    pub start_delay: usize,
    pub devisor: usize,

    #[serde(skip)]
    delay: Option<usize>,

    #[serde(skip)]
    high: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            high: false,
            start_delay: 1,
            delay: None,
            devisor: 1,
        }
    }
}

impl Module for Clock {
    fn get_root(&self) -> CellCoord {
        self.root
    }

    fn set_root(&mut self, root: CellCoord) {
        self.root = root;
    }

    fn get_pins(&self) -> Vec<Pin> {
        vec![Pin::new(0, 0, self.high, "CLK", false)]
    }

    fn clock(&mut self, _time: f64) {
        if self.delay.is_none() {
            self.delay = Some(self.start_delay);
        }

        if let Some(delay) = &mut self.delay {
            if *delay > 0 {
                *delay -= 1;
                return;
            }

            *delay = self.devisor;
            self.high = !self.high;
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub module: Clock,
    pub camera: Camera,
    pub update_self: Callback<(bool, CellCoord, Option<ConcreteModule>)>,
    pub edit_mode: bool,
}

#[function_component(ClockComponent)]
pub fn clock_component(props: &Props) -> Html {
    let Props {
        module,
        camera,
        update_self,
        edit_mode,
    } = props;

    let show_settings = use_state(|| false);

    let delay_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |e: String| {
            update_self.emit((
                true,
                module.get_root(),
                Some(ConcreteModule::Clock(Clock {
                    start_delay: e.parse::<usize>().unwrap_or(1).max(1),
                    ..(module)
                })),
            ));
        })
    };

    let devisor_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |e: String| {
            update_self.emit((
                true,
                module.get_root(),
                Some(ConcreteModule::Clock(Clock {
                    devisor: e.parse::<usize>().unwrap_or(1).max(1),
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
            <StandardPin pins={module.get_pins()} />
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
                                    key="start-delay"
                                    label="Start Delay"
                                    on_change={delay_on_change}
                                    value={format!("{}", module.start_delay)}
                                    width={24.0}
                                />
                                <TextInput
                                    key="devisor"
                                    label="Devisor"
                                    on_change={devisor_on_change}
                                    value={format!("{}", module.devisor)}
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
