use glam::{IVec2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    dom::DomIntervalHooks,
    module::{ModuleAlignment, ModuleMount},
    session::Session,
    wgl2::RenderContext,
};

pub struct YewViewport {
    pub session: Session,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
}

pub enum Msg {
    SetSession(Session),
    RawInput(RawInput),
    Render(f64),
}

pub enum RawInput {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyPressed(KeyboardEvent),
}

impl YewViewport {
    fn draw(&mut self, time: f64) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();

        // Maintain HTML Canvas size and context viewport.
        let w = canvas.client_width() as u32;
        let h = canvas.client_height() as u32;

        if w != canvas.width() || h != canvas.height() {
            canvas.set_width(w);
            canvas.set_height(h);
        }

        self.session.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );

        if let Some(render_context) = &mut self.render_context {
            render_context.draw(time, &self.session).unwrap_throw();
        }
    }
}

impl Component for YewViewport {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            session: Session::new(),
            canvas: NodeRef::default(),
            render_context: None,
            dom_events: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RawInput(raw_input) => {
                self.session.camera.handle_input_event(&raw_input);
                self.session.brush.handle_input_event(
                    &mut self.session.active_buffer,
                    &self.session.camera,
                    &raw_input,
                );
                false
            }
            Msg::SetSession(session) => {
                self.session = session;
                false
            }
            Msg::Render(time) => {
                self.draw(time);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx
            .link()
            .callback(|e| Msg::RawInput(RawInput::MouseDown(e)));
        let onmouseup = ctx.link().callback(|e| Msg::RawInput(RawInput::MouseUp(e)));
        let onmousemove = ctx
            .link()
            .callback(|e| Msg::RawInput(RawInput::MouseMove(e)));
        let onwheel = ctx
            .link()
            .callback(|e| Msg::RawInput(RawInput::MouseWheelEvent(e)));
        let onkeypress = ctx
            .link()
            .callback(|e| Msg::RawInput(RawInput::KeyPressed(e)));

        let modules = self
            .session
            .modules
            .iter()
            .map(|m| {
                let align = match m.align {
                    ModuleAlignment::UpperLeft => {
                        let offset = self.session.camera.project_cell_coord_to_screen_point(
                            CellCoord(IVec2::new(m.root.0.x, m.root.0.y + 1)),
                            false,
                        );
                        format!("left:{}px;top:{}px;", offset.x, offset.y)
                    }
                    ModuleAlignment::UpperRight => {
                        let offset = self.session.camera.project_cell_coord_to_screen_point(
                            CellCoord(IVec2::new(m.root.0.x + 1, m.root.0.y + 1)),
                            true,
                        );
                        format!("right:{}px;top:{}px;", offset.x, offset.y)
                    }
                };

                html! {
                    <div class="lp-module-container" style={align}>
                        <ModuleMount root={CellCoord(IVec2::ZERO)} module={m.module.clone()} />
                    </div>
                }
            })
            .collect::<Html>();

        html! {
            <div class="lp-viewport">
                <div class="lp-viewport-root">{ modules }</div>
                <canvas
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {onwheel}
                    {onkeypress}
                    ref={self.canvas.clone()}
                    tabindex={0}
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        self.render_context = Some(RenderContext::new(canvas).unwrap_throw());

        let link = ctx.link().clone();
        self.dom_events = Some(
            DomIntervalHooks::new(move |time| {
                link.send_message(Msg::Render(time));
            })
            .unwrap_throw(),
        );
    }
}
