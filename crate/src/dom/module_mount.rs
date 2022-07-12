use yew::prelude::*;

use crate::{coords::CellCoord, modules::Pin, wgl2::Camera};

/// The Yew wrapper around a Module's DOM node (what it's `view` function spits out) to correctly
/// position and scale the module.
pub struct ModuleMount;

#[derive(Properties)]
pub struct ModuleProps {
    pub root: CellCoord,
    pub pins: Vec<Pin>,
    #[prop_or(Camera::new())]
    pub camera: Camera,
    pub module_html: Html,
}

/// Force the mount to fully re-render every single frame.
/// TODO: Find a better solution to this.
impl PartialEq for ModuleProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Component for ModuleMount {
    type Message = ();
    type Properties = ModuleProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut root = ctx.props().root;

        // DOM and our y-coordinate space is inverted. So jump up one cell.
        root.0.y += 1;

        let pixel_offset = ctx.props().camera.project_cell_coord_to_screen_point(root);

        let root_transform_style = format!(
            "
            position: fixed;
            transform: translate({:.2}px, {:.2}px) scale({});
            ",
            pixel_offset.x,
            pixel_offset.y,
            1.0 / ctx.props().camera.scale
        );

        let pin_html = ctx
            .props()
            .pins
            .iter()
            .map(|pin| {
                html! {
                    <div
                        key={format!("{}", pin.coord_offset.0)}
                        style={format!(
                            "
                            position: absolute;
                            transform: translate({:.2}px, {:.2}px) translate({}, 0);
                            padding-right: 6px;
                            padding-left: 3px;
                            font-family: consolas;
                            font-size: 1.2em;
                            font-weight: 100;
                            line-height: 1.5em;
                            pointer-events: none;
                            ",
                            pin.coord_offset.0.x as f32 * 22.0,
                            -pin.coord_offset.0.y as f32 * 22.0,
                            if pin.right_align { "100%" } else { "-100%" }
                        )}>
                        {pin.label.to_owned()}
                    </div>
                }
            })
            .collect::<Html>();

        html! {
            <div style={root_transform_style}>
                {pin_html}
                <div style="position: absolute;">
                    {ctx.props().module_html.clone()}
                </div>
            </div>
        }
    }
}
