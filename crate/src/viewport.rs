use glam::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    blueprint::Blueprint,
    brush::{Brush, ToolType},
    buffer::Buffer,
    buffer_mask::BufferMask,
    dom::{DomIntervalHooks, ModuleMount, RawInput},
    execution_context::ExecutionContext,
    utils::Selection,
    wgl2::{Camera, RenderContext},
};

pub struct Viewport {
    pub active_buffer: Buffer,
    pub active_mask: BufferMask,
    pub execution_context: Option<ExecutionContext>,
    pub selection: Option<Selection>,
    pub camera: Camera,
    pub time: f64,
    brush: Brush,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
    on_edit_callback: Option<js_sys::Function>,
}

pub enum Msg {
    SetOnEditCallback(js_sys::Function),
    SetBlueprintPartial(Blueprint),
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

        self.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );

        if let Some(render_context) = &mut self.render_context {
            let buffer = self
                .brush
                .ephemeral_buffer
                .as_ref()
                .unwrap_or(&self.active_buffer);

            render_context
                .draw(time, buffer, &self.active_mask, &self.camera)
                .unwrap_throw();
        }
    }
}

impl Component for Viewport {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active_buffer: Buffer::default(),
            active_mask: BufferMask::default(),
            execution_context: None,
            selection: None,
            camera: Camera::new(),
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
                if self.camera.try_handle_input_event(&raw_input) {
                    return false;
                }

                let transaction_committed =
                    self.brush
                        .handle_input_event(&self.active_buffer, &self.camera, &raw_input);

                if transaction_committed {
                    if let Some(buffer) = self.brush.ephemeral_buffer.take() {
                        // Replace the active_buffer with the Brush's ephemeral one, since it wants
                        // to commit it's changes.
                        self.active_buffer = buffer;

                        // Notify JS.
                        if let Some(on_edit_callback) = self.on_edit_callback.as_ref() {
                            let json =
                                serde_json::to_string_pretty(&Blueprint::from(&self.active_buffer))
                                    .unwrap_throw();
                            let js_str = JsValue::from(&json);
                            let _ = on_edit_callback.call1(&JsValue::null(), &js_str);
                        }
                    }
                }
                false
            }
            Msg::SetOnEditCallback(callback) => {
                self.on_edit_callback = Some(callback);
                false
            }
            Msg::SetBlueprintPartial(blueprint) => {
                if let Some(new_buffer) = blueprint.into_buffer_from_partial(&self.active_buffer) {
                    self.active_buffer = new_buffer;
                }
                false
            }
            Msg::Render(time) => {
                // Update modules.
                for (_, module) in self.active_buffer.modules.iter_mut() {
                    module.update(time);
                }

                // Run the sim loop once.
                if let Some(execution_context) = &mut self.execution_context {
                    execution_context.step();
                    execution_context.update_buffer_mask(&mut self.active_mask);
                }

                self.draw(time);
                true
            }
            Msg::SetToolType(tool_type) => {
                self.brush.active_tool = tool_type;
                true
            }
            Msg::StartStopSim => {
                if self.execution_context.is_some() {
                    self.execution_context = None;
                } else {
                    self.execution_context =
                        Some(ExecutionContext::compile_from_buffer(&self.active_buffer));
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
        let onkeydown = ctx.link().callback(|e| Msg::RawInput(RawInput::KeyDown(e)));
        let onkeyup = ctx.link().callback(|e| Msg::RawInput(RawInput::KeyUp(e)));

        let modules_html = self
            .active_buffer
            .modules
            .values()
            .map(|m| {
                html! {
                    <ModuleMount camera={self.camera.clone()} module={m.clone()} />
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
                        if self.execution_context.is_none() { "green" } else { "darkred" })}
                        onclick={ctx.link().callback(|_| Msg::StartStopSim )}
                    >
                        { if self.execution_context.is_none() { "Start" } else { "Stop "} }
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
                    {onkeydown}
                    {onkeyup}
                    ref={self.canvas.clone()}
                    tabindex={0}
                    style={format!("cursor: {};", "cell")}
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
