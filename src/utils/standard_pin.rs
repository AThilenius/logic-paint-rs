use crate::modules::Pin;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub pins: Vec<Pin>,
    pub on_click: Option<Callback<usize>>,
}

#[function_component(StandardPin)]
pub fn standard_pin(props: &Props) -> Html {
    let Props { pins, on_click } = props;

    pins.iter()
        .enumerate()
        .map(|(i, pin)| {
            html! {
                <div
                    key={u64::from(pin.coord_offset)}
                    style={format!(
                        "transform: translate({}px, {}px);",
                        pin.coord_offset.0.x as f32 * 31.25,
                        -pin.coord_offset.0.y as f32 * 31.25,
                    )}
                >
                    {
                        if pin.output_high {
                            html! {
                                <div class="lp-cell-center">
                                    <div class="lp-pin-output-div" />
                                </div>
                            }
                        } else {
                            html!()
                        }
                    }
                    <div
                        class={classes!(
                            "lp-cell-center",
                            on_click.clone().map(|_| "lp-pointer-events")
                        )}
                        onclick={
                            let on_click = on_click.clone();
                            if let Some(on_click) = on_click {
                                Some(Callback::from(move |_| { on_click.emit(i); }))
                            } else {
                                None
                            }
                        }
                    >
                        <div class="lp-pin-div" />
                    </div>
                    <div
                        class={classes!("lp-cell-center")}
                        style={format!(
                            "transform: translate({}px, 0);",
                            if pin.right_align { "31.25" } else { "-31.25" }
                        )}
                    >
                        {pin.label.to_owned()}
                    </div>
                </div>
            }
        })
        .collect::<Html>()
}
