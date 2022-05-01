use glam::IVec2;
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Alignment, ModuleData, TogglePinComponent},
    wgl2::Camera,
};

/// The Yew wrapper around a Module's DOM node (what it's `view` function spits out) to correctly
/// position and scale the module.
pub struct ModuleMount;

#[derive(Properties)]
pub struct ModuleProps {
    #[prop_or(Camera::new())]
    pub camera: Camera,

    pub module: ModuleData,
    // I forgot why this needed to be an option?
    // pub module: Option<Rc<RefCell<ModuleData>>>,
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
        let anchor = ctx.props().module.get_anchor();

        // The cell we chose to align to changed depending on alignment (because in this context the
        // cells is the infinitely small crosshair where cells join up). Right we are alight right,
        // then we move one cell right, and if we are aligned top we move one cell up.
        let align_right =
            anchor.align == Alignment::TopRight || anchor.align == Alignment::BottomRight;
        let align_top = anchor.align == Alignment::TopRight || anchor.align == Alignment::TopLeft;
        let cell_coord = CellCoord(IVec2::new(
            anchor.root.0.x + if align_right { 1 } else { 0 },
            anchor.root.0.y + if align_top { 1 } else { 0 },
        ));

        // We also do a CSS translation by -100% for right aligned and -100% for bottom aligned.
        // This is needed to align the correct corner of the module node.
        let local_translation = match anchor.align {
            Alignment::TopLeft => "",
            Alignment::TopRight => "translate(-100%, 0)",
            Alignment::BottomRight => "translate(-100%, -100%)",
            Alignment::BottomLeft => "translate(0, -100%)",
        };

        // Project that cell into viewport coords (browser pixels). This operation essentially
        // encompasses both the "world" and "view" part of the projection.
        let pixel_offset = ctx
            .props()
            .camera
            .project_cell_coord_to_screen_point(cell_coord);

        // The transform origin needs to be set for scaling, otherwise it will scale around the
        // centerpoint of the node.
        let transform_origin = format!(
            "{} {}",
            if align_top { "top" } else { "bottom" },
            if align_right { "right" } else { "left" }
        );

        // Select the correct Yew component.
        let module_component = match &ctx.props().module {
            ModuleData::TogglePin(data_rc) => {
                html! { <TogglePinComponent data={data_rc.clone()} /> }
            }
        };

        html! {
            <div style={
                format!("
                    position: fixed;
                    transform-origin: {};
                    transform: translate({:.2}px, {:.2}px) {} scale({});
                    border: 1px solid red;
                    color: white;
                ",
                transform_origin,
                pixel_offset.x,
                pixel_offset.y,
                local_translation,
                1.0 / ctx.props().camera.scale)}>
                {module_component}
            </div>
        }
    }
}
