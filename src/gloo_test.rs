use futures::StreamExt;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

use crate::{dom::ElementEventHooks, log};

// pub struct OnClick {
//     receiver: mpsc::UnboundedReceiver<()>,
//     // Automatically removed from the DOM on drop!
//     listener: EventListener,
// }

// impl OnClick {
//     pub fn new(target: &EventTarget) -> Self {
//         let (sender, receiver) = mpsc::unbounded();

//         // Attach an event listener
//         let listener = EventListener::new(&target, "click", move |_event| {
//             sender.unbounded_send(()).unwrap_throw();
//         });

//         Self {
//             receiver,
//             listener,
//         }
//     }
// }

// impl Stream for OnClick {
//     type Item = ();
//     type Error = ();

//     fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
//         self.receiver.poll().map_err(|_| unreachable!())
//     }
// }

#[wasm_bindgen]
pub struct GlooTest {
    pub sim_tick: usize,
    pub frame_tick: usize,
    _hooks: ElementEventHooks,
}

#[wasm_bindgen]
impl GlooTest {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let (hooks, mut receiver) = ElementEventHooks::new(&canvas).unwrap_throw();

        let receiver = async move {
            while let Some(event) = receiver.next().await {
                log!("Event: {:#?}", event);
            }
        };

        spawn_local(receiver);

        let gloo_test = Self {
            sim_tick: 0,
            frame_tick: 0,
            _hooks: hooks,
        };
        gloo_test
    }
}

impl Drop for GlooTest {
    fn drop(&mut self) {
        log!("Dropped");
    }
}
