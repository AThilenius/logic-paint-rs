use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;

/// Hooks both the animation loop and a 'work pulling' loop for doing simulation in. Work in the
/// simulation loop should be short, which allows for yielding back to the main thread for rendering
/// at a consistent framerate.
pub struct DomIntervalHooks {
    // Animation frame
    cancel_animation: Rc<RefCell<bool>>,
}

impl DomIntervalHooks {
    pub fn new<F>(mut callback: F) -> Result<Self, JsValue>
    where
        F: FnMut(f64) + 'static,
    {
        let cancel_animation = Rc::new(RefCell::new(false));

        // The closure needs to self-reference to keep registering itself. That requires an Rc.
        let closure: Rc<RefCell<Option<Closure<dyn FnMut(_)>>>> = Rc::new(RefCell::new(None));
        let closure_c = closure.clone();

        // Because the request_animation_frame is self-referential, we have to let it run the
        // next frame otherwise we leak memory when we call cancel_animation(). Kind of shitty.
        let cancel = cancel_animation.clone();

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

            callback(
                time.as_f64()
                    .expect("Failed to convert JS time value to f64")
                    / 1000.0,
            );
        }) as Box<dyn FnMut(_)>));

        // Schedule the first frame.
        window()
            .unwrap()
            .request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

        Ok(Self { cancel_animation })
    }
}

impl Drop for DomIntervalHooks {
    fn drop(&mut self) {
        *self.cancel_animation.borrow_mut() = true;
    }
}
