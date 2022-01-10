use std::collections::HashMap;

use crate::{buffer::Buffer, coords::CellCoord};

pub struct CompilerResults {
    traces: Vec<Vec<Atom>>,
    trace_lookup_by_atom: HashMap<Atom, usize>,
    gates: Vec<Gate>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Atom {
    pub coord: CellCoord,
    pub part: CellPart,
}

#[repr(usize)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum CellPart {
    MetalAndVia = 0,
    Gate = 1,
    SiLeftUp = 2,
    SiRightDown = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct Gate {
    pub is_npn: bool,
    pub base_trace: usize,
    pub left_ec_trace: usize,
    pub right_ec_trace: usize,
}

impl CompilerResults {
    pub fn from_buffer(buffer: &Buffer) -> CompilerResults {
        let mut res = CompilerResults {
            // The 0 trace is reserved to mean the 'null' trace.
            traces: vec![vec![]],
            trace_lookup_by_atom: HashMap::new(),
            gates: vec![],
        };

        res
    }
}
