use crate::{
    socket::Socket,
    substrate::{
        buffer::Buffer,
        compiler::{Atom, CellPart, CompilerResults},
        mask::{Mask, MASK_BYTE_LEN},
    },
};

pub struct ExecutionContext {
    pub compiler_results: CompilerResults,
    pub sockets: Vec<Socket>,
    pub max_ticks_per_clock: usize,
    pub buffer_mask: Mask,
    pub state: SimState,
    pub is_mid_clock_cycle: bool,
    first_tick: bool,
}

#[derive(Default)]
pub struct SimState {
    pub tick_count: usize,
    pub clock_count: usize,
    pub gate_states: Vec<bool>,
    pub trace_states: Vec<bool>,
}

impl ExecutionContext {
    pub fn compile_from_buffer(buffer: &Buffer) -> Self {
        let compiler_results = CompilerResults::from_buffer(&buffer);
        let gate_states = vec![false; compiler_results.gates.len()];
        let trace_states = vec![false; compiler_results.traces.len()];

        Self {
            compiler_results,
            sockets: buffer.sockets.clone(),
            max_ticks_per_clock: 100_000,
            buffer_mask: Default::default(),
            state: SimState {
                tick_count: 0,
                clock_count: 0,
                gate_states,
                trace_states,
            },
            is_mid_clock_cycle: false,
            first_tick: true,
        }
    }

    pub fn clock_once(&mut self) {
        if !self.is_mid_clock_cycle {
            self.run_begin_clock_cycle();
        }

        for _ in 0..self.max_ticks_per_clock {
            if !self.run_tick_once() {
                break;
            }
        }

        self.run_complete_clock_cycle();
    }

    pub fn tick_once(&mut self) {
        if !self.is_mid_clock_cycle {
            self.run_begin_clock_cycle();
        }

        let change = self.run_tick_once();

        if !change {
            self.run_complete_clock_cycle();
        }
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

    /// Starts a clock cycle by resetting trace states and re-polling module outputs.
    fn run_begin_clock_cycle(&mut self) {
        // Gate states roll over from the previous step, trace states are reset each step.
        for state in self.state.trace_states.iter_mut() {
            *state = false;
        }

        // Collect sockets for INPUT state.
        if !self.first_tick {
            for socket in &self.sockets {
                for pin in &socket.pins {
                    let trace = *self
                        .compiler_results
                        .trace_lookup_by_atom
                        .get(&Atom {
                            coord: pin.cell_coord,
                            part: CellPart::Metal,
                        })
                        .expect("Failed to find associated trace from Atom");

                    self.state.trace_states[trace] |= pin.si_input_high;
                }
            }
        }

        self.first_tick = false;
        self.is_mid_clock_cycle = true;
    }

    #[inline]
    fn run_tick_once(&mut self) -> bool {
        let mut change = false;

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

        self.state.tick_count += 1;
        change
    }

    fn run_complete_clock_cycle(&mut self) {
        // Update gate states
        for (i, gate) in self.compiler_results.gates.iter().enumerate() {
            let base = self.state.trace_states[gate.base_trace];

            self.state.gate_states[i] = if gate.is_npn { base } else { !base };
        }

        for socket in &mut self.sockets {
            for pin in &mut socket.pins {
                let trace = *self
                    .compiler_results
                    .trace_lookup_by_atom
                    .get(&Atom {
                        coord: pin.cell_coord,
                        part: CellPart::Metal,
                    })
                    .expect("Failed to get associated trace from Atom");

                if pin.trigger && pin.si_output_high != self.state.trace_states[trace] {
                    socket.pending_update = true;
                }

                pin.si_output_high = self.state.trace_states[trace];
            }

            socket.invoke_update_callback();
        }

        self.state.clock_count += 1;
        self.is_mid_clock_cycle = false;
    }
}
