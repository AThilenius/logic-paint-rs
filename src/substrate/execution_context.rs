use crate::{
    coords::CellCoord,
    module::{Module, ModuleGpioHandle},
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart, CompilerResults},
        mask::{Mask, MASK_BYTE_LEN},
    },
};

pub type ModuleHandle = usize;
pub type BondWireHandle = isize;

pub struct ExecutionContext {
    pub compiler_results: CompilerResults,
    pub modules: Vec<Box<dyn Module>>,
    // Compacted list of all bond wires.
    pub bond_wires: Vec<BondWire>,
    // Index by trace to get bond wires.
    pub bonded_traces: Vec<BondWireHandle>,
    // Pending module triggers, indexed by ModuleHandle. When the inner vec is empty, the module
    // has no pending triggers.
    pub pending_module_triggers: Vec<Vec<ModuleGpioHandle>>,
    pub max_ticks_per_clock: usize,
    pub buffer_mask: Mask,
    pub state: SimState,
}

#[derive(Default)]
pub struct SimState {
    pub micro_ticks: usize,
    pub ticks: usize,
    pub gate_states: Vec<bool>,
    pub trace_states: Vec<bool>,
}

#[derive(Clone)]
pub struct BondWire {
    // The trace this bond wire ultimately attaches.
    pub trace: usize,

    // The cell coordinate of the socket.
    pub cell_coord: CellCoord,

    // A handle to the module that owns the GPIO.
    pub module_handle: ModuleHandle,

    // A handle to the GPIO within the module.
    pub gpio_handle: ModuleGpioHandle,
}

pub enum CompilerError {
    MissingBondWire(String),
}

impl ExecutionContext {
    pub fn compile_from_buffer(
        buffer: &Buffer,
        mut modules: Vec<Box<dyn Module>>,
    ) -> Result<Self, CompilerError> {
        let compiler_results = CompilerResults::from_buffer(&buffer);
        let gate_states = vec![false; compiler_results.gates.len()];
        let trace_states = vec![false; compiler_results.traces.len()];

        // Create bond wires (links between module GPIOs and Buffer Sockets
        let mut bond_wires = vec![];
        let mut bonded_traces = vec![-1; compiler_results.traces.len()];

        for (module_handle, module) in modules.iter_mut().enumerate() {
            for (gpio_handle, gpio) in module.get_gpios_mut().iter().enumerate() {
                let socket = buffer
                    .sockets
                    .iter()
                    .find(|s| s.name == gpio.name)
                    .ok_or_else(|| CompilerError::MissingBondWire(gpio.name.clone()))?;

                let Some(&trace) = compiler_results.trace_lookup_by_atom.get(&Atom {
                    coord: socket.cell_coord,
                    part: CellPart::Metal,
                }) else {
                    continue;
                };

                bonded_traces[trace] = bond_wires.len() as isize;
                bond_wires.push(BondWire {
                    trace,
                    cell_coord: socket.cell_coord,
                    module_handle,
                    gpio_handle,
                });
            }
        }

        Ok(Self {
            compiler_results,
            modules,
            bond_wires,
            bonded_traces,
            pending_module_triggers: vec![],
            max_ticks_per_clock: 100_000,
            buffer_mask: Default::default(),
            state: SimState {
                micro_ticks: 0,
                ticks: 0,
                gate_states,
                trace_states,
            },
        })
    }

    pub fn tick_once(&mut self) {
        // Starts a single tick (one transistor propagation-delay) which is made up of resetting
        // traces to low, propagating high traces through the stable gate network via micro-ticks,
        // then computing new gate states based on the trace values.
        for state in self.state.trace_states.iter_mut() {
            *state = false;
        }

        // Collect modules for input (from modules into the substrate) state. We skip this on the
        // first ever tick to set a consistent gate state.
        if self.state.ticks > 0 {
            for bond_wire in &self.bond_wires {
                self.state.trace_states[bond_wire.trace] |= self.modules[bond_wire.module_handle]
                    .get_gpios_mut()[bond_wire.gpio_handle]
                    .si_input_high;
            }
        }

        // Propagate trace high states through gates (who's state (open/closed) is already known).
        // Trace states are reset each tick and high states are fully propagated through the net in
        // micro-ticks. This is necessary because our transistors don't have a source/drain, they
        // are bi-directional, and a high state on one side drives a high state on the other,
        // regardless of what source it's connected to.
        //
        // Just to be really clear (for myself in the future) this only propagates high states, and
        // only one micro-tick (we aren't done computing a single tick until micro-ticks are
        // stable. And they will always become stable, because gates aren't changing state (ie the
        // graph is stable).
        loop {
            let mut change = false;
            for (i, gate) in self.compiler_results.gates.iter().enumerate() {
                // If the gate isn't conducting, ignore it as it can't effect the other trace.
                if !self.state.gate_states[i] {
                    continue;
                }

                // Get trace states
                let left = self.state.trace_states[gate.left_ec_trace];
                let right = self.state.trace_states[gate.right_ec_trace];

                // Nothing to update here, both traces are already low or both high.
                if left == right {
                    continue;
                }

                // One of the two traces is going high this micro-tick, so mark the change.
                change |= true;

                // If left is going high
                if !left {
                    self.state.trace_states[gate.left_ec_trace] = true;
                } else {
                    self.state.trace_states[gate.right_ec_trace] = true;
                }
            }

            self.state.micro_ticks += 1;

            if !change {
                break;
            }
        }

        // Update gate states
        for (i, gate) in self.compiler_results.gates.iter().enumerate() {
            let base = self.state.trace_states[gate.base_trace];
            self.state.gate_states[i] = if gate.is_npn { base } else { !base };
        }

        // Reset pending module triggers
        for triggers in &mut self.pending_module_triggers {
            triggers.clear();
        }

        // Update output (from the substrate to module) states and mark modules that have pending
        // triggers.
        for bond_wire in &self.bond_wires {
            let gpio =
                &mut self.modules[bond_wire.module_handle].get_gpios_mut()[bond_wire.module_handle];
            let trace_state = self.state.trace_states[bond_wire.trace];

            // If the state changes and the GPIO is a trigger
            if gpio.trigger && trace_state != gpio.si_output_high {
                self.pending_module_triggers[bond_wire.module_handle].push(bond_wire.gpio_handle);
            }

            // Set or clear state
            gpio.si_output_high = trace_state;
        }

        self.state.ticks += 1;
    }

    pub fn update_buffer_mask(&mut self) {
        for (chunk_coord, cell_part_to_traces) in self
            .compiler_results
            .trace_to_cell_part_index_by_chunk
            .iter()
        {
            let chunk = self.buffer_mask.get_or_create_chunk_mut(*chunk_coord);
            for index in cell_part_to_traces {
                let i = index.cell_index_in_chunk * MASK_BYTE_LEN;
                let cell_slice = &mut chunk.cells[i..i + MASK_BYTE_LEN];
                cell_slice[0] = if self.state.trace_states[index.metal_trace] {
                    1
                } else {
                    0
                };
                cell_slice[1] = if self.state.trace_states[index.si_trace] {
                    1
                } else {
                    0
                };
                cell_slice[2] = if self.state.trace_states[index.left_ec_trace] {
                    1
                } else {
                    0
                };
                cell_slice[3] = if self.state.trace_states[index.right_ec_trace] {
                    1
                } else {
                    0
                };
            }
        }
    }
}
