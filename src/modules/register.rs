use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::{coords::CellCoord, modules::Pin};

use super::{Alignment, Anchor};

// Register module pinout
// [0]     Enable (active high, tri-state output)
// [1]     Write (low = read, high = write)
// [3]     Clk (rising edge)
// [4..]   Bus (N bit)

// Full read cycles
// - Set En. Await clock. Read bus.

// Full write cycles
// - Set En, Write, set data on bus. Await clock.

// Enable: activates the register module. If write is high, the bus will be driven by the module, if
// write is low then the bus will be read on the next Clk rising edge. The bus is floating when
// enable is false, and clk is ignored.

// Write: puts the module into write mode. The next edge of Clk will result in the data (instantly)
// being committed. En and Write can both be cleared on the subsequent tick.

// Clk: rising-edge triggered. Ignored completely if either enable or write are low.

// Bus: N-bit little-endian data bus with tri-state (floating) buffers. The bus will be drive only
// when En is high and Write is low.

// Timing: timing diagrams can be omitted as the module is 'magic' and has 0 tick delays. For
// example, setting En high and Write low will immediately (before the next tick) cause the bus to
// be driven with the memory cell's value. Data is also atomic, there is no way to read an erroneous
// value from a cell because of a race condition.

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RegisterData {
    pub anchor: Anchor,

    // Pins
    pub en: Pin,
    pub write: Pin,
    pub clk: Pin,
    pub bus: Vec<Pin>,

    // State
    pub data: Vec<bool>,
    pub prev_clk: bool,
}

impl RegisterData {
    pub fn new(root: IVec2, bus_width: usize) -> Self {
        Self {
            anchor: Anchor {
                root: CellCoord(root),
                align: Alignment::BottomLeft,
            },
            en: Pin::new(root + IVec2::new(0, 0)),
            write: Pin::new(root + IVec2::new(0, 2)),
            clk: Pin::new(root + IVec2::new(0, 4)),
            bus: Pin::new_repeating(root + IVec2::new(2, 0), IVec2::new(0, 1), bus_width),
            data: vec![false; bus_width],
            prev_clk: false,
        }
    }

    pub fn reset(&mut self) {
        // TODO: Pretty sure I need to clear more values here...
        self.prev_clk = false;

        for val in self.data.iter_mut() {
            *val = false;
        }

        for pin in self.bus.iter_mut() {
            pin.output_high = false;
        }
    }

    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn get_pins(&self) -> Vec<Pin> {
        let mut pins = vec![self.en.clone(), self.write.clone(), self.clk.clone()];
        pins.extend(self.bus.iter().cloned());
        pins
    }

    pub fn set_input_pins(&mut self, states: &Vec<bool>) {
        let en = states[0];
        let write = states[1];
        let clk = states[2];

        // If En is low, then clear the bus and ignore everything else.
        if !en {
            for pin in self.bus.iter_mut() {
                pin.output_high = false;
            }

            return;
        }

        // If write is low (reading) then we can ignore the clk and immediately write value to the
        // bus.
        if !write {
            for (pin, val) in self.bus.iter_mut().zip(self.data.iter()) {
                pin.output_high = *val;
            }

            return;
        }

        // For writes, trigger on rising edge.
        if !self.prev_clk && clk {
            self.data.copy_from_slice(&states[3..3 + self.bus.len()]);

            return;
        }
    }
}

pub struct RegisterComponent;

#[derive(Properties)]
pub struct RegisterProps {
    pub data: Rc<RefCell<RegisterData>>,
}

impl PartialEq for RegisterProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub enum Msg {}

impl Component for RegisterComponent {
    type Message = Msg;
    type Properties = RegisterProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let data = ctx.props().data.borrow();
        let mut val = 0;
        for i in 0..data.data.len() {
            if data.data[(data.data.len() - 1) - i] {
                val += 1 << i;
            }
        }

        html! {
            <div
                style={
                    format!("
                            height: 44px;
                            width: 44px;
                            border: 1ps solid gray;
                            text-align: center;
                            justify-content: middle;
                        ",
                    )
                }
            >
                {format!("{}", val)}
            </div>
        }
    }
}
