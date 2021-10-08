use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, MessageChannel, MessagePort};

use crate::log;

/// Hooks both the animation loop and a 'work pulling' loop for doing simulation in. Work in the
/// simulation loop should be short, which allows for yielding back to the main thread for rendering
/// at a consistent framerate.
pub struct DomIntervalHooks {
    // Animation frame
    cancel_animation: Rc<RefCell<bool>>,

    // Simulation loop
    closure_simulation_step: Option<Closure<dyn FnMut(JsValue)>>,
    channel_simulation_port: Option<MessagePort>,
}

pub trait DomIntervalTarget {
    fn animation_frame(&mut self, time: f64);
    fn simulate_step(&mut self) -> bool;
}

impl DomIntervalHooks {
    pub fn new(target: Rc<RefCell<dyn DomIntervalTarget>>) -> Result<DomIntervalHooks, JsValue> {
        let mut dom_interval_hooks = Self {
            cancel_animation: Rc::new(RefCell::new(false)),
            closure_simulation_step: None,
            channel_simulation_port: None,
        };

        // This uses a MessageChannel to quickly yield to the event loop with a high callback
        // priority. We cannot use setTimeout because it has a 4ms minimum and we need to yield at a
        // much higher frequency for smooth rendering.
        {
            let rc = target.clone();
            let channel = MessageChannel::new().unwrap();
            let p1 = channel.port1();
            let p1_c = p1.clone();
            let p2 = channel.port2();
            let closure = Closure::wrap(Box::new(move |_: JsValue| {
                let step_again = rc.borrow_mut().simulate_step();
                if step_again {
                    // Yield and continue ASAP.
                    p1_c.post_message(&JsValue::NULL).unwrap();
                }
            }) as Box<dyn FnMut(_)>);
            p2.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            dom_interval_hooks.closure_simulation_step = Some(closure);
            dom_interval_hooks.channel_simulation_port = Some(p1);
        }

        {
            let rc = target.clone();
            let port = dom_interval_hooks.channel_simulation_port.clone().unwrap();
            // The closure needs to self-reference to keep registering itself. That requires an Rc.
            let closure: Rc<RefCell<Option<Closure<dyn FnMut(_)>>>> = Rc::new(RefCell::new(None));
            let closure_c = closure.clone();

            // Because the request_animation_frame is self-referential, we have to let it run the
            // next frame otherwise we leak memory when we call cancel_animation(). Kind of shitty.
            let cancel = dom_interval_hooks.cancel_animation.clone();

            *closure.borrow_mut() = Some(Closure::wrap(Box::new(move |time: JsValue| {
                if *cancel.borrow() {
                    // Drop our handle to this closure so that it will get cleaned up once we
                    // return. This is the only sane way to cleanup without leaking the closure
                    // memory because of the self-reference.
                    let _ = closure_c.borrow_mut().take();
                    return;
                }

                // Schedule another animation frame.
                window()
                    .unwrap()
                    .request_animation_frame(
                        closure_c
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .ok();

                rc.borrow_mut().animation_frame(time.as_f64().unwrap());

                // Trigger a simulation step at least once per frame. This acts to 'poll' the
                // handler for if it wants to start simulating.
                port.post_message(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(_)>));

            // Schedule the first frame.
            window().unwrap().request_animation_frame(
                closure.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            )?;
        }

        Ok(dom_interval_hooks)
    }
}

impl Drop for DomIntervalHooks {
    fn drop(&mut self) {
        *self.cancel_animation.borrow_mut() = true;

        if let Some(port) = self.channel_simulation_port.take() {
            port.set_onmessage(None);
        }
        drop(self.closure_simulation_step.take());
        log!("DomIntervalHooks dropped");
    }
}
