use crate::compiler::CompilerResults;

pub struct ExecutionContext {
    ir: CompilerResults,
    previous_state: SimState,
}

struct SimState {
    pub gate_states: Vec<bool>,
    pub trace_states: Vec<bool>,
    // pub pin_module_states: Vec<PinModuleState>,
    // pub pin_states: Vec<Vec<PinState>>,
}
