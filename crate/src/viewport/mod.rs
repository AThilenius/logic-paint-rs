pub mod brush;
pub mod buffer;
pub mod buffer_mask;
pub mod compiler;
pub mod execution_context;
pub mod input;

use std::collections::HashMap;

use glam::Vec2;
use gloo::{events::EventListener, utils::document};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    blueprint::Blueprint,
    dom::{DomIntervalHooks, ModuleMount, RawInput},
    utils::Selection,
    viewport::{brush::*, buffer::Buffer, execution_context::ExecutionContext, input::InputState},
    wgl2::{Camera, RenderContext},
};

pub struct Viewport {
    pub active_buffer: Buffer,
    pub ephemeral_buffer: Option<Buffer>,
    pub selection: Selection,
    pub camera: Camera,
    pub time: f64,
    pub input_state: InputState,
    registers: HashMap<String, Buffer>,
    mouse_follow_buffer: Option<Buffer>,
    mode: Mode,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
    on_edit_callback: Option<js_sys::Function>,
    request_clipboard: Option<js_sys::Function>,
    set_clipboard: Option<js_sys::Function>,
    event_hooks: Vec<EventListener>,
}

pub enum Msg {
    None,
    SetJsCallbacks {
        on_edit_callback: js_sys::Function,
        request_clipboard: js_sys::Function,
        set_clipboard: js_sys::Function,
    },
    SetBlueprintPartial(Blueprint),
    SetClipboard(Blueprint),
    RawInput(RawInput),
    Render(f64),
    SetFocus(bool),
}

pub enum Mode {
    /// (ESC) Default starting mode, accessed from any other mode with ESC.
    /// - Denoted by the cell-cursor (Excel style)
    /// - Only mode where module anchors are visible
    /// - Same selection keybinds as Excel. Clicking/Dragging selected a range. Holding Shift adds
    ///   to the selection. Holding Ctrl removes from the selection.
    /// - Hovering a trace highlights the conductive path
    /// - Double-clicking a trace selects the conductive path cells
    /// - VSCode::OnCopy copies the selected cells and modules, with the root being what ever cell
    ///   was last under the mouse at that time.
    /// - VSCode::OnPaste pastes into a 'cursor follow' buffer, next mouse click commits it to
    ///   active
    Visual,

    /// (F) Paints metal and vias.
    /// LMB: paint
    /// RMB || Shift+LMB: Via
    /// Ctrl+... to remove
    PaintMetallic,

    /// (D) Paints doped silicon
    /// LMB: paint N
    /// RMB || Shift+LMB paint P
    /// Ctrl+... to erase any type & mosfets
    PaintSi,

    /// (E) Enters execution mode
    /// (R): Run (for now just one clock per frame)
    /// (C): Enter manual-mode, clocks once.
    /// (T): Enter manual-mode, ticks once.
    /// (P): Enter manual-mode
    Execute(Execution),
}

pub struct Execution {
    pub manual: bool,
    pub context: ExecutionContext,
}

impl Viewport {
    fn draw(&mut self, time: f64) {
        self.time = time;
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();

        // Handle execution.
        if let Mode::Execute(execution) = &mut self.mode {
            if !execution.manual {
                // Update modules.
                self.active_buffer.clock_modules(time);
                execution.context.clock_once();
            }
            execution.context.update_buffer_mask();
        }

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

        // Redraw the mouse-follow buffer to the ephemeral buffer each frame.
        if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
            let mut buffer = self.active_buffer.clone();
            buffer.paste_at(self.input_state.cell, mouse_follow_buffer);
            self.ephemeral_buffer = Some(buffer);
        }

        if let Some(render_context) = &mut self.render_context {
            let buffer = self
                .ephemeral_buffer
                .as_ref()
                .unwrap_or(&self.active_buffer);

            let mask = if let Mode::Execute(execution) = &self.mode {
                Some(&execution.context.buffer_mask)
            } else {
                None
            };

            render_context
                .draw(time, buffer, &self.selection, mask, &self.camera)
                .unwrap_throw();
        }
    }

    fn dispatch_input_state(&mut self) {
        // Handle cursor-follow before anything else.
        if let Some(render_context) = &self.render_context {
            render_context.set_cursor_coord(self.input_state.cell);
        }

        // Let the camera take all events beyond that.
        if self.camera.handle_input(&self.input_state) {
            return;
        }

        // Check if a named register was clicked (we use this in multiple places).
        let named_register_clicked = "1234567890*"
            .chars()
            .map(|c| c.to_string())
            .filter(|c| self.input_state.key_clicked == *c)
            .next();

        // Keybinds: Esc => Visual, D => PaintSi, F => PaintMetallic
        if self.input_state.key_code_clicked == "Escape" {
            self.mode = Mode::Visual;
            self.selection = Default::default();
            self.mouse_follow_buffer = None;
            self.ephemeral_buffer = None;
        } else if self.input_state.key_code_clicked == "KeyQ" {
            self.mode = Mode::PaintSi;
            self.selection = Default::default();
            self.mouse_follow_buffer = None;
        } else if self.input_state.key_code_clicked == "KeyW" {
            self.mode = Mode::PaintMetallic;
            self.selection = Default::default();
            self.mouse_follow_buffer = None;
        } else if self.input_state.key_code_clicked == "KeyE"
            && !matches!(self.mode, Mode::Execute(..))
        {
            self.mode = Mode::Execute(Execution {
                manual: true,
                context: ExecutionContext::compile_from_buffer(&self.active_buffer),
            });
            self.selection = Default::default();
            self.mouse_follow_buffer = None;
        }

        match &mut self.mode {
            Mode::Visual => {
                if let Some(mouse_follow_buffer) = self.mouse_follow_buffer.clone() {
                    // Handle placing the mouse follow buffer.
                    if self.input_state.primary_clicked {
                        self.active_buffer
                            .paste_at(self.input_state.cell, &mouse_follow_buffer);

                        self.notify_js_on_change();
                    }

                    // Right click (and ESC) clears the mouse follow buffer.
                    if self.input_state.secondary {
                        self.mouse_follow_buffer = None;
                        self.ephemeral_buffer = None;
                    }

                    // Hitting KeyS + any of the named register keys will save the mouse-follow
                    // buffer into the named register.
                    if self.input_state.key_codes_down.contains("KeyS") {
                        if let Some(named_register) = &named_register_clicked {
                            // If it's the clipboard register, also set the clipboard.
                            if named_register == "*" {
                                self.notify_js_set_clipboard(&mouse_follow_buffer);
                            } else {
                                self.registers
                                    .insert(named_register.clone(), mouse_follow_buffer.clone());
                            }
                            self.selection = Default::default();
                        }
                    } else {
                        // Otherwise override the mouse-follow buffer with the newly selected
                        // register, if it exists.
                        if let Some(named_register) = &named_register_clicked {
                            if let Some(buffer) = self.registers.get(named_register) {
                                self.mouse_follow_buffer = Some(buffer.clone());
                            }
                        }
                    }
                } else {
                    if self.input_state.primary {
                        if let Some(drag) = self.input_state.drag {
                            self.selection = Selection::from_rectangle_inclusive(
                                drag.start,
                                self.input_state.cell,
                            );
                        }
                    } else if self.input_state.secondary {
                        self.selection = Default::default();
                    }

                    // Delete selection
                    if self.input_state.key_code_clicked == "KeyD" {
                        if !self.input_state.shift {
                            self.mouse_follow_buffer = Some(
                                self.active_buffer
                                    .clone_selection(&self.selection, self.input_state.cell),
                            );
                        }
                        clear_both_in_selection(&mut self.active_buffer, &self.selection);
                        self.selection = Default::default();
                        self.notify_js_on_change();
                    }

                    // Yank selection to mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyY" {
                        self.mouse_follow_buffer = Some(
                            self.active_buffer
                                .clone_selection(&self.selection, self.input_state.cell),
                        );
                        self.selection = Default::default();
                    }

                    // Hitting KeyS + any of the named register keys will save the selected cells
                    // into the named register.
                    if self.input_state.key_codes_down.contains("KeyS") && !self.selection.is_zero()
                    {
                        if let Some(named_register) = &named_register_clicked {
                            let buffer = self
                                .active_buffer
                                .clone_selection(&self.selection, self.input_state.cell);

                            // If it's the clipboard register, also set the clipboard.
                            if named_register == "*" {
                                self.notify_js_set_clipboard(&buffer);
                            } else {
                                self.registers.insert(named_register.clone(), buffer);
                            }
                            self.selection = Default::default();
                        }
                    } else {
                        // Hitting any of the named register keys (while not holding KeyS) will load
                        // the register into the mouse-follow buffer.
                        if let Some(named_register) = named_register_clicked {
                            // If it's the clipboard register then we have to request the clipboard from
                            // JS and wait for it to come back. Sucks.
                            if named_register == "*" {
                                self.notify_js_request_clipboard();
                            } else if let Some(buffer) = self.registers.get(&named_register) {
                                self.mouse_follow_buffer = Some(buffer.clone());
                            }
                            self.selection = Default::default();
                        }
                    }
                }
            }
            Mode::PaintMetallic | Mode::PaintSi => {
                self.dispatch_paint_input_state();
            }
            Mode::Execute(Execution { manual, context }) => {
                if self.input_state.key_code_clicked == "KeyR" {
                    *manual = false;
                } else if self.input_state.key_code_clicked == "KeyC" {
                    *manual = true;
                    self.active_buffer.clock_modules(self.time);
                    context.clock_once();
                } else if self.input_state.key_code_clicked == "KeyT" {
                    *manual = true;
                    // Only clock modules if we are between clock cycles.
                    if !context.is_mid_clock_cycle {
                        self.active_buffer.clock_modules(self.time);
                    }
                    context.tick_once();
                } else if self.input_state.key_code_clicked == "KeyP" {
                    *manual = true;
                }
            }
        }

        if self.input_state.key_code_clicked == "KeyF" {
            self.active_buffer.fix_all_cells();
        }
    }

    fn dispatch_paint_input_state(&mut self) {
        // If neither button is clicked
        if !self.input_state.primary && !self.input_state.secondary {
            // Commit the ephemeral buffer if we have one.
            if let Some(buffer) = self.ephemeral_buffer.take() {
                self.active_buffer = buffer;

                // Notify JS.
                self.notify_js_on_change();
            }

            return;
        }

        // Painting generates a totally new Buffer (cloned from active) each time.
        let mut buffer = self.active_buffer.clone();

        let path = self.input_state.get_impulse_drag_path();

        // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
        // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
        if self.input_state.ctrl {
            match self.mode {
                Mode::PaintMetallic => clear_metal(&mut buffer, path),
                Mode::PaintSi => clear_si(&mut buffer, path),
                _ => {}
            }
        } else {
            // Input modes are much, much more complicated. That logic is delegated to it's own file
            // because they are so stupid-complicated.
            let mut from = None;

            for cell_coord in path {
                match self.mode {
                    Mode::PaintMetallic => {
                        // Primary paints metal, secondary places a Via (only once).
                        if self.input_state.primary {
                            draw_metal(&mut buffer, from, cell_coord);
                        } else if self.input_state.secondary {
                            draw_via(&mut buffer, from, cell_coord);
                        }
                    }
                    Mode::PaintSi => {
                        // Primary paints N-type, secondary paints P-type.
                        if self.input_state.primary {
                            draw_si(&mut buffer, from, cell_coord, true);
                        } else {
                            draw_si(&mut buffer, from, cell_coord, false);
                        }
                    }
                    _ => {}
                }
                from = Some(cell_coord);
            }
        }

        self.ephemeral_buffer = Some(buffer);
    }

    fn notify_js_on_change(&self) {
        if let Some(on_edit_callback) = self.on_edit_callback.as_ref() {
            let json =
                serde_json::to_string_pretty(&Blueprint::from(&self.active_buffer)).unwrap_throw();
            let js_str = JsValue::from(&json);
            let _ = on_edit_callback.call1(&JsValue::null(), &js_str);
        }
    }

    fn notify_js_request_clipboard(&self) {
        if let Some(request_clipboard) = self.request_clipboard.as_ref() {
            let _ = request_clipboard.call0(&JsValue::null());
        }
    }

    fn notify_js_set_clipboard(&self, buffer: &Buffer) {
        if let Some(set_clipboard) = self.set_clipboard.as_ref() {
            let json = serde_json::to_string_pretty(&Blueprint::from(buffer)).unwrap_throw();
            let js_str = JsValue::from(&json);
            let _ = set_clipboard.call1(&JsValue::null(), &js_str);
        }
    }
}

impl Component for Viewport {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active_buffer: Buffer::default(),
            ephemeral_buffer: None,
            selection: Default::default(),
            camera: Camera::new(),
            time: 0.0,
            input_state: Default::default(),
            registers: HashMap::new(),
            mouse_follow_buffer: None,
            mode: Mode::Visual,
            canvas: NodeRef::default(),
            render_context: None,
            dom_events: None,
            on_edit_callback: None,
            request_clipboard: None,
            set_clipboard: None,
            event_hooks: vec![],
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RawInput(raw_input) => {
                self.input_state.handle_raw_input(&self.camera, &raw_input);
                self.dispatch_input_state();
            }
            Msg::SetJsCallbacks {
                on_edit_callback,
                request_clipboard,
                set_clipboard,
            } => {
                self.on_edit_callback = Some(on_edit_callback);
                self.request_clipboard = Some(request_clipboard);
                self.set_clipboard = Some(set_clipboard);
            }
            Msg::SetBlueprintPartial(blueprint) => {
                if let Some(new_buffer) = blueprint.into_buffer_from_partial(&self.active_buffer) {
                    self.active_buffer = new_buffer;
                }
            }
            Msg::SetClipboard(blueprint) => {
                if let Some(buffer) = blueprint.into_buffer_from_partial(&Buffer::default()) {
                    self.mode = Mode::Visual;
                    self.mouse_follow_buffer = Some(buffer.clone());
                    self.registers.insert("*".to_owned(), buffer);
                }
            }
            Msg::Render(time) => {
                self.draw(time);
            }
            Msg::SetFocus(focused) => {
                if !focused {
                    self.mode = Mode::Visual;
                }
            }
            Msg::None => {}
        }

        true
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
        let oncontextmenu = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            Msg::None
        });

        let modules_html = self
            .active_buffer
            .rooted_modules
            .values()
            .map(|m| {
                let pins = m.module.borrow().get_pins();
                html! {
                    <ModuleMount
                        camera={self.camera.clone()}
                        root={m.root}
                        pins={pins}
                        module_html={m.html.clone()}
                    />
                }
            })
            .collect::<Html>();

        let mode = html!(
            <div style="font-weight: bold;">
            {
                match &self.mode {
                    Mode::Visual => html!(<span style="color: darkgreen;">{"Visual"}</span>),
                    Mode::PaintMetallic => html!(<span style="color: gray;">{"Metal"}</span>),
                    Mode::PaintSi => html!(<span style="color: rgb(255, 0, 255);">{
                        "Silicon"
                    }</span>),
                    Mode::Execute(Execution { manual, context }) => {
                        html!(
                            <span style={
                                format!("color: {};", if *manual { "orange" } else { "green" })
                            }>
                            <div>{if *manual { "Execution Paused" } else { "Running" }}</div>
                            <div>{format!("Ticks: {}", context.state.tick_count)}</div>
                            <div>{
                                format!("Fundamental Clocks: {}", context.state.clock_count)
                            }</div>
                            </span>
                        )
                    }
                }
            }
            </div>
        );

        html! {
            <div class="lp-viewport">
                <canvas
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {oncontextmenu}
                    {onwheel}
                    ref={self.canvas.clone()}
                    style={
                        let cursor = {
                            if self.input_state.key_codes_down.contains("Space") {
                                "grabbing"
                            } else {
                                match self.mode {
                                    Mode::Visual => "cell",
                                    Mode::PaintMetallic | Mode::PaintSi => "crosshair",
                                    Mode::Execute(..) => "default",
                                }
                            }
                        };

                        format!("cursor: {};", cursor)
                    }
                />
                <div class="lp-info-panel">
                    {mode}
                    <div>{format!("Cursor: {}", self.input_state.cell.0)}</div>
                    <div>
                        {"Selection: "}
                        {
                            if self.selection.is_zero() {
                                "None".to_owned()
                            } else {
                                format!(
                                    "{} -> {}",
                                    self.selection.lower_left.0,
                                    self.selection.upper_right.0
                                )
                            }
                        }
                    </div>
                </div>
                <span
                    style={
                        if matches!(self.mode, Mode::PaintSi)
                            || matches!(self.mode, Mode::PaintMetallic) {
                            "pointer-events: none;"
                        } else {
                            ""
                        }
                    }
                >
                    { modules_html }
                </span>
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

        let window = window().unwrap();
        let document = document();

        let link = ctx.link().clone();
        let key_down = EventListener::new(&document, "keydown", move |event| {
            let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            link.send_message(Msg::RawInput(RawInput::KeyDown(event.clone())));
        });

        let link = ctx.link().clone();
        let key_up = EventListener::new(&document, "keyup", move |event| {
            let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            link.send_message(Msg::RawInput(RawInput::KeyUp(event.clone())));
        });

        let link = ctx.link().clone();
        let focus = EventListener::new(&window, "focus", move |_| {
            link.send_message(Msg::SetFocus(true));
        });

        let link = ctx.link().clone();
        let blur = EventListener::new(&window, "blur", move |_| {
            link.send_message(Msg::SetFocus(false));
        });

        self.event_hooks = vec![key_down, key_up, focus, blur];
    }
}
