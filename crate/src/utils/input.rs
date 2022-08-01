use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::HtmlInputElement;
use web_sys::InputEvent;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub disabled: Option<bool>,
    pub value: Option<String>,
    pub on_change: Callback<String>,
    pub label: Option<String>,
    pub width: Option<f32>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().expect("Failed to dyn_into case InputEvent");
    let event_target = event
        .target()
        .expect("Failed to get target from input event");
    let target: HtmlInputElement = event_target
        .dyn_into()
        .expect("Failed to dyn_into HtmlInputElement");
    target.value()
}

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &Props) -> Html {
    let Props {
        disabled,
        value,
        on_change,
        label,
        width,
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
        <div>
            {
                if let Some(label) = label {
                    html!(<label key={102} for="input-tag" style="margin-right: 10px;">{label}</label>)
                } else {
                    html!()
                }
            }
            <input
                id="input-tag"
                type="text"
                style={
                    if let Some(width) = width {
                        format!("width: {}px", width)
                    } else {
                        "".to_owned()
                    }
                }
                {disabled}
                {value}
                {oninput}
            />
        </div>
    }
}
