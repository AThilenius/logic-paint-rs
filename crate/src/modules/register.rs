use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

use crate::modules::{Module, Pin};

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

pub struct PinMap {
    pub en: Pin,
    pub write: Pin,
    pub clk: Pin,
    pub bus: Vec<Pin>,
}

impl From<&Vec<Pin>> for PinMap {
    fn from(vec: &Vec<Pin>) -> Self {
        Self {
            en: vec[0].clone(),
            write: vec[1].clone(),
            clk: vec[2].clone(),
            bus: vec[3..].to_owned(),
        }
    }
}

impl Into<Vec<Pin>> for &PinMap {
    fn into(self) -> Vec<Pin> {
        let mut pins = vec![self.en.clone(), self.write.clone(), self.clk.clone()];
        pins.extend(self.bus.iter().cloned());
        pins
    }
}

pub struct Register {
    pub pins: PinMap,
    pub data: Vec<bool>,
}

impl Register {
    pub fn new(bus_width: usize) -> Self {
        Self {
            pins: PinMap {
                en: Pin::new(0, 0, false, "EN", false),
                write: Pin::new(0, -1, false, "WRT", false),
                clk: Pin::new(0, -2, false, "CLK", false),
                bus: Pin::new_repeating((2, 0).into(), (0, -1).into(), bus_width, "D", true),
            },
            data: vec![false; bus_width],
        }
    }
}

impl Module for Register {
    fn get_pins(&self) -> Vec<Pin> {
        (&self.pins).into()
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        let mut pins: PinMap = pins.into();

        // If En is low, then clear the bus and ignore everything else.
        if !pins.en.input_high {
            for pin in pins.bus.iter_mut() {
                pin.output_high = false;
            }

            self.pins = pins;
            return;
        }

        // If write is low (reading) then we can ignore the clk and immediately write the value to
        // the bus.
        if !pins.write.input_high {
            for (pin, val) in pins.bus.iter_mut().zip(self.data.iter()) {
                pin.output_high = *val;
            }

            self.pins = pins;
            return;
        }

        // For writes, trigger on rising edge.
        if !self.pins.clk.input_high && pins.clk.input_high {
            self.data = pins.bus.iter().map(|p| p.input_high).collect();

            self.pins = pins;
            return;
        }
    }
}

pub struct RegisterComponent;

#[derive(Properties)]
pub struct RegisterProps {
    pub data: Rc<RefCell<Register>>,
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

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
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
            <div style={
                format!("
                        height: {}px;
                        width: {}px;
                        border: 1px solid red;
                        text-align: center;
                        justify-content: middle;
                    ",
                    data.data.len() * 22,
                    22 * 3
                )
            }>
                {format!("{}", val)}
            </div>
        }
    }
}
