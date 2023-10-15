// A byte-addressable RAM module with 16-bit addresses and magic 'instantaneous' reads and writes.
//
// Data can be both read and saved in either 8-bit or 16-bit modes.
//
// In 8-bit mode (SIZE = high) the addresses is taken as a byte-aligned 16-bit address, and the bus
// is treated as an 8-bit value that is sign-extended to 16-bits during reads.
//
// In 16-bit value (SIZE = low) the address must be word-aligned (ie ADDR[0] must be 0), and the
// full bus is used. Bytes are stored in memory little-endian (ie the Least Significant Byte is
// stored at a lower memory address).
//
// Instead of simulating real RAM, where both reads and writes take an indeterminate amount of time
// and the CPU must wait for a READY flag, I've chose to instead just make this module 'magic', in
// that it can both read and write data instantaneously (as far as the simulation cares). Adding
// artificial latency does nothing but slow down the simulation, and incorporating a ready-wait
// state into a multi-cycle architecture isn't particularly interesting, so it doesn't feel too much
// like cheating.
//
// Addressing & Data:
//  - [16] ADDR: Address select
//  - [16] BUS: shared bus
// Control (active high)
//  - SIZE (high = 8 bit, low = 16 bit)
//  - OUT.EN:
//    if SIZE = BYTE
//      BUS[7:0] = MEM[ADDR]
//    else if ADDR[0] == 0
//      BUS[7:0] = MEM[ADDR]
//      BUS[15:8] = MEM[ADDR + 1]
//    else
//      Undefined (word addressing must be word-aligned).
//  - WRITE:
//    if SIZE = BYTE
//      MEM[ADDR] = BUS[7:0]
//    else if ADDR[0] == 0
//      MEM[ADDR] = BUS[7:0]
//      MEM[ADDR + 1] = BUS[15:8]
//    else
//      Undefined (word addressing must be word-aligned).
use glam::IVec2;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Module, Pin},
    utils::{
        cell_offset::CellOffset, local_cell_offset::LocalCellOffset, standard_pin::StandardPin,
    },
    wgl2::Camera,
};

use super::ConcreteModule;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub root: CellCoord,
    pub spacing: usize,
    pub data: Vec<u8>,

    #[serde(skip)]
    pub addr: u16,

    #[serde(skip)]
    pub byte_addressed: bool,

    #[serde(skip)]
    pub out_en: bool,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            root: CellCoord(IVec2::ZERO),
            data: vec![0_u8; u16::MAX as usize],
            spacing: 10,
            addr: 0,
            byte_addressed: false,
            out_en: false,
        }
    }
}

impl Module for Memory {
    fn get_root(&self) -> CellCoord {
        return self.root;
    }

    fn set_root(&mut self, root: CellCoord) {
        self.root = root;
    }

    fn get_pins(&self) -> Vec<Pin> {
        let size = Pin::new(0, -1, false, "SIZE", false);
        let out_en = Pin::new(0, 0, false, "OUT.EN", false);
        let write = Pin::new(0, 1, false, "WRITE", false);

        let addr = Pin::new_repeating(
            (3 + 1 + (self.spacing * 15) as i32, 0).into(),
            (-(self.spacing as i32), 0).into(),
            16,
            "A",
            true,
        );

        let mut data = Pin::new_repeating(
            (3 + (self.spacing * 15) as i32, 0).into(),
            (-(self.spacing as i32), 0).into(),
            16,
            "D",
            false,
        );

        if self.out_en {
            if self.byte_addressed {
                let byte = self.data[self.addr as usize];
                let short = byte as i8 as i16;
                for i in 0..16_usize {
                    data[i].output_high = (short >> i) & 1 > 0;
                }
            } else {
                // Address must be word-aligned. Mask the last bit to force that fact.
                let addr = (self.addr & (!1_u16)) as usize;
                // LSB is at ADDR, MSB is at ADDR+1
                let lsb = self.data[addr];
                let msb = self.data[addr + 1];

                for i in 0..8_usize {
                    data[i].output_high = (lsb >> i) & 1 > 0;
                }

                for i in 0..8_usize {
                    data[8 + i].output_high = (msb >> i) & 1 > 0;
                }
            }
        }

        let mut pins = vec![size, out_en, write];
        pins.extend(addr.into_iter());
        pins.extend(data.into_iter());

        pins
    }

    fn set_pins(&mut self, pins: &Vec<Pin>) {
        self.byte_addressed = pins[0].input_high;
        self.out_en = pins[1].input_high;
        let write = pins[2].input_high;

        self.addr = 0_u16;
        for i in 0..16_usize {
            if pins[3 + i].input_high {
                self.addr |= 1 << i;
            }
        }

        if write {
            if self.byte_addressed {
                let mut data = 0_u8;
                for i in 0..8_usize {
                    if pins[3 + 16 + i].input_high {
                        data |= 1 << i;
                    }
                }
                self.data[self.addr as usize] = data;
            } else {
                let mut lsb = 0_u8;
                for i in 0..8_usize {
                    if pins[3 + 16 + i].input_high {
                        lsb |= 1 << i;
                    }
                }
                self.data[self.addr as usize] = lsb;

                let mut msb = 0_u8;
                for i in 0..8_usize {
                    if pins[3 + 16 + 8 + i].input_high {
                        msb |= 1 << i;
                    }
                }
                self.data[self.addr as usize + 1] = msb;
            }
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub module: Memory,
    pub camera: Camera,
    pub update_self: Callback<(bool, CellCoord, Option<ConcreteModule>)>,
    pub edit_mode: bool,
}

#[function_component(MemoryComponent)]
pub fn memory_component(props: &Props) -> Html {
    let Props {
        module,
        camera,
        update_self,
        edit_mode,
    } = props;

    let show_settings = use_state(|| false);

    let delete_on_change = {
        let module = module.clone();
        let update_self = update_self.clone();
        Callback::from(move |_| {
            update_self.emit((true, module.get_root(), None));
        })
    };

    html! {
        <CellOffset camera={camera.clone()} root={module.root} >
            <StandardPin pins={module.get_pins()} />
            {
                if *edit_mode {
                    html! {
                        <div
                            class={classes!("lp-module-edit-mode-div", "lp-pointer-events")}
                            onclick={
                                let show_settings = show_settings.clone();
                                Callback::from(move |_| show_settings.set(!*show_settings))
                            }
                        >
                            {"⚙"}
                        </div>
                    }
                } else {
                    html!()
                }
            }
            {
                if *edit_mode && *show_settings {
                    html! {
                        <LocalCellOffset amount={IVec2::new(1, 0)}>
                            <div class={classes!("lp-settings-panel", "lp-pointer-events")}>
                                <div style="
                                    background: red;
                                    margin-bottom: 4px;
                                    padding: 0 2px;"
                                    onclick={delete_on_change}>
                                    {"DEL"}
                                </div>
                                // <TextInput
                                //     label="Spacing"
                                //     on_change={spacing_on_change}
                                //     value={format!("{}", module.spacing)}
                                //     width={24.0}
                                // />
                            </div>
                        </LocalCellOffset>
                    }
                } else {
                    html!()
                }
            }
        </CellOffset>
    }
}
