use std::mem::forget;

use dom::{DomIntervalHooks, ElementEventHooks};
use viewport::Viewport;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};
use yew::prelude::*;

mod brush;
mod dom;
mod sim;
mod substrate;
mod utils;
mod viewport;
mod wgl2;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

enum Msg {
    AddOne,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, value: 0 }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // let document = web_sys::window().unwrap().document().unwrap();
    // let canvas = document.get_element_by_id("wasm-canvas").unwrap();
    // let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

    // let substrate_viewport = unwrap_or_log_and_return!(Viewport::from_canvas(canvas.clone()));
    // let dom_interval_hooks =
    //     unwrap_or_log_and_return!(DomIntervalHooks::new(substrate_viewport.clone()));
    // let element_event_hooks = unwrap_or_log_and_return!(ElementEventHooks::new(
    //     canvas.dyn_into::<HtmlElement>().unwrap(),
    //     substrate_viewport.clone()
    // ));

    // forget(dom_interval_hooks);
    // forget(element_event_hooks);

    yew::start_app::<Model>();
}
