use glam::IVec2;

use super::SimTickParams;

#[derive(Debug, Clone)]
pub enum PinModule {
    ConstVal {
        cell_loc: IVec2,
        high: bool,
    },
    Clock {
        cell_loc: IVec2,
        interval: usize,
        name: String,
        starts_high: bool,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Pin {
    pub cell_loc: IVec2,
    pub name: String,
}

impl PinModule {
    pub fn get_pins(&self) -> Vec<Pin> {
        match self {
            PinModule::ConstVal { cell_loc, high } => vec![Pin {
                name: if *high {
                    "VCC".to_owned()
                } else {
                    "GND".to_owned()
                },
                cell_loc: *cell_loc,
            }],
            PinModule::Clock { name, cell_loc, .. } => vec![Pin {
                name: name.clone(),
                cell_loc: *cell_loc,
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub enum PinModuleState {
    ConstVal {
        high: bool,
    },
    Clock {
        module: PinModule,
        next_edge: usize,
        high: bool,
    },
}

impl PinModuleState {
    pub fn instantiate(pin_module: &PinModule) -> PinModuleState {
        match pin_module {
            &PinModule::ConstVal { high, .. } => PinModuleState::ConstVal { high },
            &PinModule::Clock {
                interval,
                starts_high,
                ..
            } => PinModuleState::Clock {
                module: pin_module.clone(),
                next_edge: interval,
                high: starts_high,
            },
        }
    }

    pub fn update_pin_state_inputs(
        &mut self,
        pin_states: &mut Vec<PinState>,
        params: &SimTickParams,
    ) {
        match self {
            PinModuleState::Clock {
                module: PinModule::Clock { interval, .. },
                next_edge,
                high,
            } => {
                if params.tick >= *next_edge {
                    *high = !*high;
                    *next_edge = *interval + params.tick;
                }

                pin_states[0].input_high = *high;
            }
            PinModuleState::ConstVal { high } => {
                pin_states[0].input_high = *high;
            }
            _ => unreachable!("Module type should always match module state type"),
        }
    }

    pub fn handle_pin_state_outputs(
        &mut self,
        _pin_states: &Vec<PinState>,
        _sim_state: &SimTickParams,
    ) {
        // Clock pins are like the Honey Badger, it doesn't give a shit. Other pin modules will.
    }
}

#[derive(Debug, Default, Clone)]
pub struct PinState {
    pub input_high: bool,
    pub output_high: bool,
}
