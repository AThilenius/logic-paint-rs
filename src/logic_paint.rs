use std::{cell::RefCell, rc::Rc};

use futures::StreamExt;
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

use crate::{
    coords::CellCoord,
    dom::{DomIntervalHooks, ElementEventHooks},
    session::SerdeSession,
    upc::{Bit, UnpackedCell, UPC},
    viewport::Viewport,
};

/// Public facing JS API. This mostly just wraps a Viewport, and dispatched DOM events to it.
#[wasm_bindgen]
pub struct LogicPaint {
    viewport: Rc<RefCell<Viewport>>,
    element_hooks: ElementEventHooks,
    dom_hooks: DomIntervalHooks,
}

#[wasm_bindgen]
impl LogicPaint {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let (element_hooks, mut el_recv) = ElementEventHooks::new(&canvas).unwrap_throw();
        let (dom_hooks, mut dom_recv) = DomIntervalHooks::new().unwrap_throw();

        let viewport = Rc::new(RefCell::new(Viewport::from_canvas(canvas).unwrap_throw()));

        // Dev
        viewport
            .borrow_mut()
            .session
            .active_buffer
            .transaction_begin();
        viewport
            .borrow_mut()
            .session
            .active_buffer
            .transact_set_cell(
                CellCoord::from((-1, -1)),
                UnpackedCell {
                    si_n: true,
                    si_dir_left: true,
                    si_dir_up: true,
                    metal: true,
                    metal_dir_right: true,
                    metal_dir_down: true,
                    via: true,
                    ..Default::default()
                }
                .into(),
            );
        viewport
            .borrow_mut()
            .session
            .active_buffer
            .transaction_commit(true);

        // Handle DOM events
        {
            let viewport = viewport.clone();
            spawn_local(async move {
                while let Some(event) = dom_recv.next().await {
                    viewport.borrow_mut().handle_dom_interval_event(event);
                }
            });
        }

        // Handle element events
        {
            let viewport = viewport.clone();
            spawn_local(async move {
                while let Some(event) = el_recv.next().await {
                    viewport.borrow_mut().handle_element_input_event(event);
                }
            });
        }

        Self {
            element_hooks,
            dom_hooks,
            viewport,
        }
    }

    pub fn set_mode_editor(&mut self) {}

    pub fn set_mode_sim_paused(&mut self) {}

    pub fn set_mode_sim_free_run(
        &mut self,
        max_ticks_per_frame: Option<u64>,
        max_ticks_per_second: Option<u64>,
    ) {
    }

    pub fn set_session_from_string(&mut self, data: &str) -> Option<String> {
        if !data.starts_with("LP-S-V1:") {
            return Some("String is not valid LogicPaint Session Version 1 data.".to_string());
        }

        let compressed_bytes = {
            if let Ok(compressed_bytes) = base64::decode(&data[8..]) {
                compressed_bytes
            } else {
                return Some(
                    "Data is not valid Base64. Are you missing some characters at the end?"
                        .to_string(),
                );
            }
        };

        let bytes = {
            if let Ok(bytes) = decompress_to_vec(&compressed_bytes) {
                bytes
            } else {
                return Some(
                    "Failed to decompress session data. Das ist nicht so gut...".to_string(),
                );
            }
        };

        let session = {
            let res = bincode::deserialize::<SerdeSession>(&bytes);

            if let Err(err) = res {
                return Some(err.to_string());
            }

            res.unwrap()
        };

        self.viewport.borrow_mut().set_session((&session).into());

        None
    }

    pub fn get_session_as_string(&mut self) -> String {
        let bytes =
            bincode::serialize(&SerdeSession::from(&self.viewport.borrow().session)).unwrap_throw();
        let compressed_bytes = compress_to_vec(&bytes, 6);
        format!("LP-S-V1:{}", base64::encode(compressed_bytes))
    }
}
