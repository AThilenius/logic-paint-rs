use crate::substrate::execution_context::SimState;

pub type ModuleGpioHandle = usize;

/// Connects a module GPIO to a substrate socket.
pub struct ModuleGpio {
    /// The handle used to communicate with this GPIO, generally just the index of the GPIO in the
    /// Vec storing the module's GPIOs.
    pub handle: ModuleGpioHandle,

    /// The human-readable name for this GPIO
    pub name: String,

    /// The 'bond wire' that connects this GPIO to a substrate Socket.
    pub bonding: Option<String>,

    /// When set to true, the module will be notified (at the completion of the tick) when the
    /// `si_output_high` transitions from false to true. There is no way to trigger on a falling
    /// edge, just use an inverter infront of the socket.
    pub trigger: bool,

    // Set to true when the substrate is driving the pin high.
    pub si_output_high: bool,

    // Set to true by the module, to drive the substrate socket to high.
    pub si_input_high: bool,
}

pub trait Module {
    /// Provides the human-readable name for this module.
    fn get_name(&self) -> &str;

    /// Resets the module back to it's pre-execution state.
    fn reset(&mut self);

    /// Provides a list of GPIOs, each with a unique handle that can be referenced by the module.
    fn get_gpios_mut(&mut self) -> &mut Vec<ModuleGpio>;

    /// Called when one or more GPIOs have been triggered. By the time this is called, the
    /// ModuleGpio instance has already been updated.
    fn trigger(&mut self, sim_state: &SimState, handles: Vec<ModuleGpioHandle>) {
        let _ = handles;
        let _ = sim_state;
    }

    /// Called each clock cycle.
    fn clock(&mut self, sim_state: &SimState) {
        let _ = sim_state;
    }
}

// ==== Clock module ==============================================================================
pub struct ModuleClock {
    gpios: Vec<ModuleGpio>,
}

impl Default for ModuleClock {
    fn default() -> Self {
        Self {
            gpios: vec![ModuleGpio {
                handle: 0,
                name: "CLK".to_string(),
                bonding: None,
                trigger: false,
                si_output_high: false,
                si_input_high: false,
            }],
        }
    }
}

impl Module for ModuleClock {
    fn get_name(&self) -> &str {
        "Clock"
    }

    fn reset(&mut self) {
        self.gpios[0].si_input_high = false;
    }

    #[inline(always)]
    fn get_gpios_mut(&mut self) -> &mut Vec<ModuleGpio> {
        &mut self.gpios
    }

    fn clock(&mut self, sim_state: &SimState) {
        self.gpios[0].si_input_high = sim_state.ticks % 2 == 0;
    }
}
