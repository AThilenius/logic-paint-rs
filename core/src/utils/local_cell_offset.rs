use glam::IVec2;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub amount: IVec2,
    pub children: Children,
}

#[function_component(LocalCellOffset)]
pub fn local_cell_offset(props: &Props) -> Html {
    let Props { amount, children } = props;

    html! {
        <div
            style={format!("
                position: absolute;
                transform: translate({}px, {}px);
            ", amount.x as f32 * 31.25, -amount.y as f32 * 31.25)}
        >
            {for children.iter()}
        </div>
    }
}
