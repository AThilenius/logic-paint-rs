use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, MouseEvent, WheelEvent};

/// Hooks a DOM Element and feeds mouse/keyboard events to a target trait object. The target cannot
/// be dropped before this struct is dropped.
pub struct ElementEventHooks {
    element: HtmlElement,
    closure_mouse_down: Option<Closure<dyn FnMut(MouseEvent)>>,
    closure_mouse_move: Option<Closure<dyn FnMut(MouseEvent)>>,
    closure_mouse_up: Option<Closure<dyn FnMut(MouseEvent)>>,
    closure_mouse_scroll: Option<Closure<dyn FnMut(WheelEvent)>>,
}

pub trait ElementEventTarget {
    fn on_input_event(&mut self, event: ElementInputEvent);
}

#[derive(Clone, Debug)]
pub enum ElementInputEvent {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
}

impl ElementEventHooks {
    pub fn new(
        html_element: HtmlElement,
        target: Rc<RefCell<dyn ElementEventTarget>>,
    ) -> Result<Self, JsValue> {
        let mut element_event_target = Self {
            element: html_element.clone(),
            closure_mouse_down: None,
            closure_mouse_up: None,
            closure_mouse_move: None,
            closure_mouse_scroll: None,
        };

        // Register event handlers
        {
            let rc = target.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                rc.borrow_mut()
                    .on_input_event(ElementInputEvent::MouseDown(event));
            }) as Box<dyn FnMut(_)>);
            html_element
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            element_event_target.closure_mouse_down = Some(closure);
        }
        {
            let rc = target.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                rc.borrow_mut()
                    .on_input_event(ElementInputEvent::MouseUp(event));
            }) as Box<dyn FnMut(_)>);
            html_element
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            element_event_target.closure_mouse_up = Some(closure);
        }
        {
            let rc = target.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                rc.borrow_mut()
                    .on_input_event(ElementInputEvent::MouseMove(event));
            }) as Box<dyn FnMut(_)>);
            html_element
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            element_event_target.closure_mouse_move = Some(closure);
        }
        {
            let rc = target.clone();
            let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
                rc.borrow_mut()
                    .on_input_event(ElementInputEvent::MouseWheelEvent(event));
            }) as Box<dyn FnMut(_)>);
            html_element
                .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
            element_event_target.closure_mouse_scroll = Some(closure);
        }

        Ok(element_event_target)
    }
}

impl Drop for ElementEventHooks {
    fn drop(&mut self) {
        if let Some(closure) = self.closure_mouse_down.take() {
            self.element
                .remove_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .ok();
        }
        if let Some(closure) = self.closure_mouse_up.take() {
            self.element
                .remove_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
                .ok();
        }
        if let Some(closure) = self.closure_mouse_move.take() {
            self.element
                .remove_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
                .ok();
        }
        if let Some(closure) = self.closure_mouse_scroll.take() {
            self.element
                .remove_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
                .ok();
        }
    }
}
