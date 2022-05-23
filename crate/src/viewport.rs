use glam::Vec2;
use miniz_oxide::deflate::compress_to_vec;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    brush::{Brush, ToolType},
    dom::{DomIntervalHooks, ModuleMount, RawInput},
    execution_context::ExecutionContext,
    session::{SerdeSession, Session},
    wgl2::RenderContext,
};

pub struct Viewport {
    pub session: Session,
    pub time: f64,
    brush: Brush,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
    on_edit_callback: Option<js_sys::Function>,
}

pub enum Msg {
    SetOnEditCallback(js_sys::Function),
    SetSession(Session),
    RawInput(RawInput),
    Render(f64),
    SetToolType(ToolType),
    StartStopSim,
}

impl Viewport {
    fn draw(&mut self, time: f64) {
        self.time = time;
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

impl Component for Viewport {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            session: Session::new(),
            time: 0.0,
            brush: Brush::new(),
            canvas: NodeRef::default(),
            render_context: None,
            dom_events: None,
            on_edit_callback: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RawInput(raw_input) => {
                self.session.camera.handle_input_event(&raw_input);
                let transaction_committed = self.brush.handle_input_event(
                    &mut self.session.active_buffer,
                    &self.session.camera,
                    &raw_input,
                );
                if transaction_committed {
                    if let Some(on_edit_callback) = self.on_edit_callback.as_ref() {
                        let bytes =
                            bincode::serialize(&SerdeSession::from(&self.session)).unwrap_throw();
                        let compressed_bytes = compress_to_vec(&bytes, 6);
                        let session_string = format!("LPS1:{}", base64::encode(compressed_bytes));
                        let js_str = JsValue::from(&session_string);
                        let _ = on_edit_callback.call1(&JsValue::null(), &js_str);
                    }
                }
                false
            }
            Msg::SetOnEditCallback(callback) => {
                self.on_edit_callback = Some(callback);
                false
            }
            Msg::SetSession(session) => {
                self.session = session;
                false
            }
            Msg::Render(time) => {
                self.session.update(time);
                self.draw(time);
                true
            }
            Msg::SetToolType(tool_type) => {
                self.brush.active_tool = tool_type;
                true
            }
            Msg::StartStopSim => {
                if self.session.execution_context.is_some() {
                    self.session.execution_context = None;
                } else {
                    self.session.execution_context = Some(ExecutionContext::compile_from_buffer(
                        &self.session.active_buffer,
                    ));
                }
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

        let modules = &self.session.active_buffer.get_modules();

        let modules_html = modules
            .iter()
            .map(|m| {
                html! {
                    <ModuleMount camera={self.session.camera.clone()} module={m.clone()} />
                }
            })
            .collect::<Html>();

        html! {
            <div class="lp-viewport">
                <div style={"
                    display: flex;
                    flex-direction: row;
                    justify-content: center;
                    position: absolute;
                    bottom: 0;
                    width: 100%;
                "}>
                    <button
                        style={format!("
                            height: 40px;
                            background: {};
                        ",
                        if self.session.execution_context.is_none() { "green" } else { "darkred" })}
                        onclick={ctx.link().callback(|_| Msg::StartStopSim )}
                    >
                        { if self.session.execution_context.is_none() { "Start" } else { "Stop "} }
                    </button>

                    <button
                        style={format!("
                            height: 40px;
                            background: {};
                        ",
                        if self.brush.active_tool == ToolType::NType { "magenta" } else { "gray" })}
                        onclick={ctx.link().callback(|_| Msg::SetToolType(ToolType::NType) )}
                    >
                        {"N-Type"}
                    </button>
                    <button
                        style={format!("
                            height: 40px;
                            background: {};
                        ",
                        if self.brush.active_tool == ToolType::PType { "#00deff" } else { "gray" })}
                        onclick={ctx.link().callback(|_| Msg::SetToolType(ToolType::PType) )}
                    >
                        {"P-Type"}
                    </button>
                    <button
                        style={format!("
                            height: 40px;
                            background: {};
                        ",
                        if self.brush.active_tool == ToolType::Metal {
                            "lightgray"
                        } else {
                            "gray"
                        })}
                        onclick={ctx.link().callback(|_| Msg::SetToolType(ToolType::Metal) )}
                    >
                        {"Metal"}
                    </button>
                    <button
                        style={format!("
                            height: 40px;
                            background: {};
                        ",
                        if self.brush.active_tool == ToolType::Via { "lightgray" } else { "gray" })}
                        onclick={ctx.link().callback(|_| Msg::SetToolType(ToolType::Via) )}
                    >
                        {"Via"}
                    </button>
                </div>
                <span>{ modules_html }</span>
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
        self.render_context = Some(RenderContext::new(canvas).unwrap());

        let link = ctx.link().clone();
        self.dom_events = Some(
            DomIntervalHooks::new(move |time| {
                link.send_message(Msg::Render(time));
            })
            .unwrap_throw(),
        );
    }
}
