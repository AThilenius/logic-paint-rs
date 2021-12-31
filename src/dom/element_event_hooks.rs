use futures::channel::mpsc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{EventTarget, KeyboardEvent, MouseEvent, WheelEvent};

/// Hooks a DOM Element and feeds mouse/keyboard events to a target trait object. The target cannot
/// be dropped before this struct is dropped.
pub struct ElementEventHooks {
    target: EventTarget,
    closure_mouse_down: Closure<dyn FnMut(MouseEvent)>,
    closure_mouse_move: Closure<dyn FnMut(MouseEvent)>,
    closure_mouse_up: Closure<dyn FnMut(MouseEvent)>,
    closure_mouse_scroll: Closure<dyn FnMut(WheelEvent)>,
    closure_key_down: Closure<dyn FnMut(KeyboardEvent)>,
}

#[derive(Clone, Debug)]
pub enum ElementInputEvent {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyPressed(KeyboardEvent),
}

impl ElementEventHooks {
    pub fn new(target: &EventTarget) -> Result<(Self, mpsc::Receiver<ElementInputEvent>), JsValue> {
        let (sender, receiver) = mpsc::channel(1_000);

        let closure_mouse_down = {
            let mut sender = sender.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                sender
                    .try_send(ElementInputEvent::MouseDown(event))
                    .unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            target
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure
        };

        let closure_mouse_up = {
            let mut sender = sender.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                sender
                    .try_send(ElementInputEvent::MouseUp(event))
                    .unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            target.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure
        };

        let closure_mouse_move = {
            let mut sender = sender.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                sender
                    .try_send(ElementInputEvent::MouseMove(event))
                    .unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            target
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure
        };

        let closure_mouse_scroll = {
            let mut sender = sender.clone();
            let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
                sender
                    .try_send(ElementInputEvent::MouseWheelEvent(event))
                    .unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            target.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
            closure
        };

        let closure_key_down = {
            let mut sender = sender.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                sender
                    .try_send(ElementInputEvent::KeyPressed(event))
                    .unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            target.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure
        };

        Ok((
            Self {
                target: target.clone(),
                closure_mouse_down,
                closure_mouse_up,
                closure_mouse_move,
                closure_mouse_scroll,
                closure_key_down,
            },
            receiver,
        ))
    }
}

impl Drop for ElementEventHooks {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback(
                "mousedown",
                self.closure_mouse_down.as_ref().unchecked_ref(),
            )
            .ok();

        self.target
            .remove_event_listener_with_callback(
                "mouseup",
                self.closure_mouse_up.as_ref().unchecked_ref(),
            )
            .ok();

        self.target
            .remove_event_listener_with_callback(
                "mousemove",
                self.closure_mouse_move.as_ref().unchecked_ref(),
            )
            .ok();

        self.target
            .remove_event_listener_with_callback(
                "wheel",
                self.closure_mouse_scroll.as_ref().unchecked_ref(),
            )
            .ok();

        self.target
            .remove_event_listener_with_callback(
                "keydown",
                self.closure_key_down.as_ref().unchecked_ref(),
            )
            .ok();
    }
}
