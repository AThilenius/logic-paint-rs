use glam::IVec2;
use yew::prelude::*;

use crate::{coords::CellCoord, wgl2::Camera};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub camera: Camera,
    pub root: CellCoord,
    pub offset: Option<IVec2>,
    pub children: Children,
}

#[function_component(CellOffset)]
pub fn cell_offset(props: &Props) -> Html {
    let Props {
        camera,
        root,
        offset,
        children,
    } = props;

    // DOM and our y-coordinate space is inverted. So jump up one cell.
    let mut root = CellCoord(root.0 + IVec2::new(0, 1));

    if let Some(offset) = offset {
        root.0 += *offset;
    }

    let pixel_offset = camera.project_cell_coord_to_screen_point(root);

    html! {
        <div
            class="lp-module-root"
            style={format!(
                "transform: translate({:.2}px, {:.2}px) scale({});",
                pixel_offset.x,
                pixel_offset.y,
                1.0 / camera.scale
            )}
        >
            {for children.iter()}
        </div>
    }
}
