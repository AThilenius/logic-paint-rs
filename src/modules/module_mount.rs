use glam::IVec2;
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Alignment, Module},
    wgl2::Camera,
};

pub struct ModuleMount;

#[derive(Properties, PartialEq)]
pub struct ModuleProps {
    #[prop_or(Camera::new())]
    pub camera: Camera,

    #[prop_or(None)]
    pub module: Option<Module>,
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
        let module = ctx.props().module.as_ref().unwrap();
        let anchor = module.get_anchor();

        match anchor.align {
            Alignment::UpperLeft => {
                let offset = ctx.props().camera.project_cell_coord_to_screen_point(
                    CellCoord(IVec2::new(anchor.root.0.x, anchor.root.0.y + 1)),
                    false,
                );
                let css = format!("left:{}px;top:{}px;", offset.x, offset.y);
                html! {
                    <div class="lp-module-container" style={css}>
                        {module.view()}
                    </div>
                }
            }
            Alignment::UpperRight => {
                let offset = ctx.props().camera.project_cell_coord_to_screen_point(
                    CellCoord(IVec2::new(anchor.root.0.x + 1, anchor.root.0.y + 1)),
                    true,
                );
                let css = format!("right:{}px;top:{}px;", offset.x, offset.y);
                html! {
                    <div class="lp-module-container" style={css}>
                        {module.view()}
                    </div>
                }
            }
        }
    }
}
