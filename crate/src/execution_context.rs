use wasm_bindgen::UnwrapThrowExt;

use crate::{
    buffer::Buffer,
    buffer_mask::{BufferMask, MASK_BYTE_LEN},
    compiler::{Atom, CellPart, CompilerResults},
    modules::{ModuleData, Pin},
};

pub struct ExecutionContext {
    compiler_results: CompilerResults,
    modules: Vec<ModuleData>,
    state: SimState,
}

#[derive(Default)]
struct SimState {
    pub step_count: usize,
    pub gate_states: Vec<bool>,
    pub trace_states: Vec<bool>,
    // pub pin_module_states: Vec<PinModuleState>,
    // pub pin_states: Vec<Vec<PinState>>,
}

impl ExecutionContext {
    pub fn compile_from_buffer(buffer: &Buffer) -> Self {
        let compiler_results = CompilerResults::from_buffer(&buffer);
        let gate_states = vec![false; compiler_results.gates.len()];
        let trace_states = vec![false; compiler_results.traces.len()];
        let modules = buffer.get_modules();

        Self {
            compiler_results,
            modules,
            state: SimState {
                step_count: 0,
                gate_states,
                trace_states,
            },
        }
    }

    pub fn step(&mut self) {
        // Gate states roll over from the previous step, trace states are reset each step.
        for state in self.state.trace_states.iter_mut() {
            *state = false;
        }

        // Pull modules for OUTPUT state (input state is updates at the end of a tick) and write
        // their value to the corresponding trace.
        for module_data in self.modules.iter() {
            for pin in module_data.get_pins() {
                let trace = *self
                    .compiler_results
                    .trace_lookup_by_atom
                    .get(&Atom {
                        coord: pin.coord,
                        part: CellPart::Metal,
                    })
                    .unwrap_throw();

                self.state.trace_states[trace] |= pin.output_high;
            }
        }

        // Propagate high signal through the graph.
        let mut change = true;
        while change {
            change = false;

            for i in 0..self.compiler_results.gates.len() {
                // If the gate isn't open, ignore it.
                if !self.state.gate_states[i] {
                    continue;
                }

                let gate = self.compiler_results.gates[i];
                let left = self.state.trace_states[gate.left_ec_trace];
                let right = self.state.trace_states[gate.right_ec_trace];
                let high = left || right;
                change |= left != right;
                self.state.trace_states[gate.left_ec_trace] = high;
                self.state.trace_states[gate.right_ec_trace] = high;
            }
        }

        // Update gate states
        for (i, gate) in self.compiler_results.gates.iter().enumerate() {
            let base = self.state.trace_states[gate.base_trace];

            self.state.gate_states[i] = if gate.is_npn { base } else { !base };
        }

        // Update module inputs. First immutably collect their values.
        let module_pin_states = self
            .modules
            .iter()
            .map(|module_data| {
                module_data
                    .get_pins()
                    .iter()
                    .map(|Pin { coord, .. }| {
                        let trace = *self
                            .compiler_results
                            .trace_lookup_by_atom
                            .get(&Atom {
                                coord: *coord,
                                part: CellPart::Metal,
                            })
                            .unwrap_throw();

                        self.state.trace_states[trace]
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Then write them to modules
        for (i, module_data) in self.modules.iter_mut().enumerate() {
            module_data.set_input_pins(&module_pin_states[i]);
        }

        self.state.step_count += 1;
    }

    pub fn update_buffer_mask(&self, buffer_mask: &mut BufferMask) {
        for (chunk_coord, cell_part_to_traces) in self
            .compiler_results
            .trace_to_cell_part_index_by_chunk
            .iter()
        {
            let chunk = buffer_mask.get_or_create_chunk_mut(*chunk_coord);
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
