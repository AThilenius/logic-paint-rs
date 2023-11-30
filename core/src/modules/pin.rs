use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::coords::CellCoordOffset;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    pub label: String,
    pub right_align: bool,

    /// The cell offset coordinate where this pin lives, relative to it's anchor.
    pub coord_offset: CellCoordOffset,

    /// Set to true if the module itself is driving the pin. Modules internally set this value,
    /// and it is subsequently read by the ExecutionContext during simulation.
    pub output_high: bool,

    /// Set to true if the module is being driven by a substrate trace. The execution context sets
    /// this value, and it is subsequently read by the module.
    pub input_high: bool,
}

impl Pin {
    pub fn new(x: i32, y: i32, output_high: bool, label: &str, right_align: bool) -> Self {
        Self {
            label: label.to_owned(),
            right_align,
            coord_offset: CellCoordOffset((x, y).into()),
            output_high,
            input_high: false,
        }
    }

    pub fn new_repeating(
        start: IVec2,
        offset: IVec2,
        n: usize,
        label_prefix: &str,
        right_align: bool,
    ) -> Vec<Pin> {
        let mut cursor = start;
        let mut pins = Vec::new();
        for i in 0..n {
            pins.push(Pin::new(
                cursor.x,
                cursor.y,
                false,
                &format!("{}{}", label_prefix, i),
                right_align,
            ));
            cursor += offset;
        }

        pins
    }
}
