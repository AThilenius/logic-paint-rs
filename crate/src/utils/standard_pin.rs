use yew::prelude::*;

use crate::modules::Pin;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub pins: Vec<Pin>,
}

#[function_component(StandardPin)]
pub fn standard_pin(props: &Props) -> Html {
    let Props { pins } = props;

    pins.iter()
        .map(|pin| {
            html! {
                <div
                    key={u64::from(pin.coord_offset)}
                    class="lp-pin-root"
                    style={format!(
                        "transform: translate({}px, {}px);",
                        pin.coord_offset.0.x as f32 * 31.25,
                        -pin.coord_offset.0.y as f32 * 31.25,
                    )}
                >
                    <div
                        class="lp-pin-label"
                        style={format!(
                            "transform: translate({}, 0);",
                            if pin.right_align { "100%" } else { "-100%" }
                        )}
                    >
                        {pin.label.to_owned()}
                    </div>
                </div>
            }
        })
        .collect::<Html>()
}
