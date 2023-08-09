use glam::IVec2;
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    modules::{Pin, RootedModule},
    wgl2::Camera,
};

/// The Yew wrapper around a Module's DOM node (what it's `view` function spits out) to correctly
/// position and scale the module.
pub struct ModuleMount {
    edit_mode: bool,
}

#[derive(Properties)]
pub struct ModuleProps {
    pub pins: Vec<Pin>,
    #[prop_or(Camera::default())]
    pub camera: Camera,
    pub module: RootedModule,
    pub edit_mode: bool,
    pub ignore_all_input: bool,
    pub notify_js: Callback<()>,
}

pub enum Msg {
    ToggleEditMode,
}

/// Force the mount to fully re-render every single frame.
/// TODO: Find a better solution to this.
impl PartialEq for ModuleProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Component for ModuleMount {
    type Message = Msg;
    type Properties = ModuleProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { edit_mode: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleEditMode => {
                self.edit_mode = !self.edit_mode;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ModuleProps {
            module,
            pins,
            camera,
            edit_mode,
            ignore_all_input,
            notify_js,
        } = ctx.props();

        // DOM and our y-coordinate space is inverted. So jump up one cell.
        let root = CellCoord(module.root.0 + IVec2::new(0, 1));
        let pixel_offset = camera.project_cell_coord_to_screen_point(root);

        let pin_html = pins
            .iter()
            .map(|pin| {
                html! {
                    <div
                        key={format!("{}", pin.coord_offset.0)}
                        class="lp-pin-root"
                        style={format!(
                            "transform: translate({:.2}px, {:.2}px);",
                            pin.coord_offset.0.x as f32 * 22.0,
                            -pin.coord_offset.0.y as f32 * 22.0,
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
            .collect::<Html>();

        module
            .module
            .borrow_mut()
            .set_edit_mode(*edit_mode && self.edit_mode);

        html! {
            <div
                class={classes!(
                    "lp-module-root",
                    if *ignore_all_input { Some("lp-no-pointer-events") } else { None },
                )}
                style={format!(
                    "transform: translate({:.2}px, {:.2}px) scale({});",
                    pixel_offset.x,
                    pixel_offset.y,
                    1.0 / ctx.props().camera.scale
                )}
            >
                {pin_html}
                <div style="position: absolute;">
                    {module.html.clone()}
                </div>
                {
                    if *edit_mode {
                        html! {
                            <div
                                class="lp-module-edit-mode-div"
                                onclick={ctx.link().callback(|_| Msg::ToggleEditMode)}
                            >
                                {"⚙"}
                            </div>
                        }
                    } else {
                        html!()
                    }
                }
            </div>
        }
    }
}
