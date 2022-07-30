use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::Event;
use web_sys::HtmlInputElement;
use web_sys::InputEvent;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub id: Option<String>,
    pub disabled: Option<bool>,
    pub value: Option<String>,
    pub on_change: Callback<String>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    web_sys::console::log_1(&target.value().into());
    target.value()
}

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &Props) -> Html {
    let Props {
        id,
        disabled,
        value,
        on_change,
    } = props.clone();

    let internal_value = use_state(|| "".to_owned());

    let oninput = {
        let value = value.clone();
        let internal_value = internal_value.clone();

        Callback::from(move |input_event: InputEvent| {
            let str = get_value_from_input_event(input_event);
            if value.is_none() {
                internal_value.set(str.clone());
            }
            on_change.emit(str);
        })
    };

    let disabled = disabled.unwrap_or_default();
    let value = value.unwrap_or((*internal_value).clone());

    html! {
        <input {id} type="text" {disabled} {value} {oninput} />
    }
}
