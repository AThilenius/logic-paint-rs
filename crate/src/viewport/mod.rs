pub mod blueprint;
pub mod brush;
pub mod buffer;
pub mod buffer_mask;
pub mod compiler;
pub mod editor_state;
pub mod execution_context;
pub mod input;
mod label_builder;

use glam::{IVec2, Vec2};
use gloo::{events::EventListener, utils::document};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    coords::CellCoord,
    dom::{DomIntervalHooks, RawInput},
    modules::{ConcreteModule, Module, ValueComponent},
    utils::Selection,
    viewport::{
        blueprint::Blueprint,
        brush::*,
        buffer::Buffer,
        editor_state::{EditorState, SerdeEditorState},
        execution_context::ExecutionContext,
        input::InputState,
    },
    wgl2::RenderContext,
};

use self::label_builder::LabelBuilder;

pub struct Viewport {
    pub editor_state: EditorState,
    pub active_buffer: Buffer,
    pub ephemeral_buffer: Option<Buffer>,
    pub time: f64,
    pub input_state: InputState,
    mouse_follow_buffer: Option<Buffer>,
    mode: Mode,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
    on_edit_callback: Option<js_sys::Function>,
    on_editor_state_callback: Option<js_sys::Function>,
    request_clipboard: Option<js_sys::Function>,
    set_clipboard: Option<js_sys::Function>,
    event_hooks: Vec<EventListener>,
}

pub enum Msg {
    None,
    SetJsCallbacks {
        on_edit_callback: js_sys::Function,
        on_editor_state_callback: js_sys::Function,
        request_clipboard: js_sys::Function,
        set_clipboard: js_sys::Function,
    },
    SetBlueprint(Blueprint),
    SetEditorState(EditorState),
    SetClipboard(Blueprint),
    RawInput(RawInput),
    Render(f64),
    SetFocus(bool),
    SetModule((CellCoord, Option<ConcreteModule>)),
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

    /// (Enter) Starts Label mode.
    /// (ESC, Enter, Primary, Secondary) Leaves label model.
    Label(LabelBuilder),

    /// (M) Module mode
    ModuleEdit(Option<ConcreteModule>),
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

        // let dpr = window().unwrap().device_pixel_ratio() as f32;
        let size = Vec2::new(w as f32, h as f32);
        // let scaled_size = size * dpr;
        self.editor_state.camera.update(size);

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
                .draw(
                    time,
                    buffer,
                    &self.editor_state.selection,
                    mask,
                    &self.editor_state.camera,
                )
                .expect("Failed to draw render context");
        }
    }

    fn dispatch_input_state(&mut self) {
        // Handle cursor-follow before anything else.
        if let Some(render_context) = &self.render_context {
            render_context.set_cursor_coord(self.input_state.cell);
        }

        // Let the camera take all events beyond that. However, we need to suppress space when in
        // label mode.
        if !(matches!(self.mode, Mode::Label(_))
            && self.input_state.key_codes_down.contains("Space"))
        {
            if self.editor_state.camera.handle_input(&self.input_state) {
                // Let JS know the camera changed.
                if let Some(editor_state_callback) = &self.on_editor_state_callback {
                    let json =
                        serde_json::to_string_pretty(&SerdeEditorState::from(&self.editor_state))
                            .expect("Failed to serialize SerdeEditorState");
                    let js_str = JsValue::from(&json);
                    let _ = editor_state_callback.call1(&JsValue::null(), &js_str);
                }

                // Then early return.
                return;
            }
        }

        // Check if a named register was clicked (we use this in multiple places).
        let named_register_clicked = "1234567890*"
            .chars()
            .map(|c| c.to_string())
            .filter(|c| self.input_state.key_clicked == *c)
            .next();

        // Escape is a global keybind, it always brings us back to Visual mode
        if self.input_state.key_code_clicked == "Escape" {
            self.mode = Mode::Visual;
            self.editor_state.selection = Default::default();
            self.ephemeral_buffer = None;
            self.mouse_follow_buffer = None;
        }

        // The rest of the keybinds only make sense when not typing a label.
        if !matches!(self.mode, Mode::Label(..)) {
            // Enter => Label, Esc => Visual, D => PaintSi, F => PaintMetallic
            if self.input_state.key_code_clicked == "Enter" {
                self.mode = Mode::Label(LabelBuilder::default());
                self.editor_state.selection = Default::default();
                self.ephemeral_buffer = None;

                // Return so that we don't send the initial enter to the builder
                return;
            } else if self.input_state.key_code_clicked == "KeyQ" {
                self.mode = Mode::PaintSi;
                self.editor_state.selection = Default::default();
                self.mouse_follow_buffer = None;
            } else if self.input_state.key_code_clicked == "KeyW" {
                self.mode = Mode::PaintMetallic;
                self.editor_state.selection = Default::default();
                self.mouse_follow_buffer = None;
            } else if self.input_state.key_code_clicked == "KeyE"
                && !matches!(self.mode, Mode::Execute(..))
            {
                self.mode = Mode::Execute(Execution {
                    manual: true,
                    context: ExecutionContext::compile_from_buffer(&self.active_buffer),
                });
                self.editor_state.selection = Default::default();
                self.mouse_follow_buffer = None;
            } else if self.input_state.key_code_clicked == "KeyM"
                && !matches!(self.mode, Mode::ModuleEdit(..))
            {
                self.mode = Mode::ModuleEdit(None);
                self.editor_state.selection = Default::default();
                self.mouse_follow_buffer = None;
                return;
            }
        }

        let mut set_mouse_follow_buffer = false;
        let mut new_mouse_follow_buffer = None;
        let mut notify_js = false;

        match &mut self.mode {
            Mode::Visual => {
                // TODO: Get rid of this clone call.
                if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
                    // Handle placing the mouse follow buffer.
                    if self.input_state.primary_clicked {
                        self.active_buffer
                            .paste_at(self.input_state.cell, &mouse_follow_buffer);

                        notify_js = true;
                    }

                    // Right click (and ESC) clears the mouse follow buffer.
                    if self.input_state.secondary {
                        set_mouse_follow_buffer = true;
                        new_mouse_follow_buffer = None;
                        self.ephemeral_buffer = None;
                    }

                    // KeyR will rotate the mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyR" {
                        set_mouse_follow_buffer = true;
                        new_mouse_follow_buffer = Some(mouse_follow_buffer.rotate_to_new());
                    }

                    // KeyM will mirror the mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyM" {
                        set_mouse_follow_buffer = true;
                        new_mouse_follow_buffer = Some(mouse_follow_buffer.mirror_to_new());
                    }

                    // Hitting KeyS + any of the named register keys will save the mouse-follow
                    // buffer into the named register.
                    if self.input_state.key_codes_down.contains("KeyS") {
                        if let Some(named_register) = &named_register_clicked {
                            // If it's the clipboard register, also set the clipboard.
                            if named_register == "*" {
                                self.notify_js_set_clipboard(&mouse_follow_buffer);
                            } else {
                                self.editor_state
                                    .registers
                                    .insert(named_register.clone(), mouse_follow_buffer.clone());
                            }
                            self.editor_state.selection = Default::default();
                        }
                    } else {
                        // Otherwise override the mouse-follow buffer with the newly selected
                        // register, if it exists.
                        if let Some(named_register) = &named_register_clicked {
                            if let Some(buffer) = self.editor_state.registers.get(named_register) {
                                set_mouse_follow_buffer = true;
                                new_mouse_follow_buffer = Some(buffer.clone());
                            }
                        }
                    }
                } else {
                    if self.input_state.primary {
                        if let Some(drag) = self.input_state.drag {
                            self.editor_state.selection = Selection::from_rectangle_inclusive(
                                drag.start,
                                self.input_state.cell,
                            );
                        }
                    } else if self.input_state.secondary {
                        self.editor_state.selection = Default::default();
                    }

                    // Delete selection
                    if self.input_state.key_code_clicked == "KeyD" {
                        if !self.input_state.shift {
                            set_mouse_follow_buffer = true;
                            new_mouse_follow_buffer = Some(self.active_buffer.clone_selection(
                                &self.editor_state.selection,
                                self.input_state.cell,
                            ));
                        }
                        clear_both_in_selection(
                            &mut self.active_buffer,
                            &self.editor_state.selection,
                        );
                        self.editor_state.selection = Default::default();
                        notify_js = true;
                    }

                    // Yank selection to mouse-follow buffer
                    if self.input_state.key_code_clicked == "KeyY" {
                        set_mouse_follow_buffer = true;
                        new_mouse_follow_buffer =
                            Some(self.active_buffer.clone_selection(
                                &self.editor_state.selection,
                                self.input_state.cell,
                            ));
                        self.editor_state.selection = Default::default();
                    }

                    // Hitting KeyS + any of the named register keys will save the selected cells
                    // into the named register.
                    if self.input_state.key_codes_down.contains("KeyS")
                        && !self.editor_state.selection.is_zero()
                    {
                        if let Some(named_register) = &named_register_clicked {
                            let buffer = self.active_buffer.clone_selection(
                                &self.editor_state.selection,
                                self.input_state.cell,
                            );

                            // If it's the clipboard register, also set the clipboard.
                            if named_register == "*" {
                                self.notify_js_set_clipboard(&buffer);
                            } else {
                                self.editor_state
                                    .registers
                                    .insert(named_register.clone(), buffer);
                            }
                            self.editor_state.selection = Default::default();
                        }
                    } else {
                        // Hitting any of the named register keys (while not holding KeyS) will load
                        // the register into the mouse-follow buffer.
                        if let Some(named_register) = named_register_clicked {
                            // If it's the clipboard register then we have to request the clipboard
                            // from JS and wait for it to come back. Sucks.
                            if named_register == "*" {
                                self.notify_js_request_clipboard();
                            } else if let Some(buffer) =
                                self.editor_state.registers.get(&named_register)
                            {
                                set_mouse_follow_buffer = true;
                                new_mouse_follow_buffer = Some(buffer.clone());
                            }
                            self.editor_state.selection = Default::default();
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
            Mode::Label(label_builder) => {
                label_builder.dispatch_input(&self.input_state);
                set_mouse_follow_buffer = true;
                new_mouse_follow_buffer = Some(label_builder.render_to_buffer(true));

                // Handle placing the text.
                if self.input_state.primary_clicked {
                    self.active_buffer.paste_at(
                        self.input_state.cell,
                        &label_builder.render_to_buffer(false),
                    );

                    notify_js = true;
                }
            }
            Mode::ModuleEdit(module) => {
                if let Some(module) = module {
                    // Click to place module
                    if self.input_state.primary_clicked {
                        let mut module = module.clone();
                        module.set_root(self.input_state.cell);

                        self.active_buffer
                            .modules
                            .insert(self.input_state.cell, module);

                        notify_js = true;
                    }
                }

                // Tab to cycle module types
                if self.input_state.key_code_clicked == "KeyM" {
                    *module = match module {
                        // Some(ConcreteModule::Clock(_)) => {
                        //     Some(ConcreteModule::Value(Default::default()))
                        // }
                        Some(ConcreteModule::Value(_)) => None,
                        // None => Some(ConcreteModule::Clock(Default::default())),
                        None => Some(ConcreteModule::Value(Default::default())),
                    };

                    let mut buffer = Buffer::default();
                    if let Some(module) = module {
                        let cell_zero = CellCoord(IVec2::ZERO);
                        let mut module = module.clone();
                        module.set_root(cell_zero);
                        buffer.modules.insert(cell_zero, module);
                    }

                    set_mouse_follow_buffer = true;
                    new_mouse_follow_buffer = Some(buffer);
                }
            }
        }

        if set_mouse_follow_buffer {
            self.mouse_follow_buffer = new_mouse_follow_buffer;
        }

        if notify_js {
            self.notify_js_on_change();
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
            let json = serde_json::to_string_pretty(&Blueprint::from(&self.active_buffer))
                .expect("Failed to serialize Blueprint");
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
            let json = serde_json::to_string_pretty(&Blueprint::from(buffer))
                .expect("Failed to serialize Blueprint");
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
            editor_state: Default::default(),
            active_buffer: Buffer::default(),
            ephemeral_buffer: None,
            time: 0.0,
            input_state: Default::default(),
            mouse_follow_buffer: None,
            mode: Mode::Visual,
            canvas: NodeRef::default(),
            render_context: None,
            dom_events: None,
            on_edit_callback: None,
            on_editor_state_callback: None,
            request_clipboard: None,
            set_clipboard: None,
            event_hooks: vec![],
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RawInput(raw_input) => {
                self.input_state
                    .handle_raw_input(&self.editor_state.camera, &raw_input);
                self.dispatch_input_state();
            }
            Msg::SetJsCallbacks {
                on_edit_callback,
                on_editor_state_callback,
                request_clipboard,
                set_clipboard,
            } => {
                self.on_edit_callback = Some(on_edit_callback);
                self.on_editor_state_callback = Some(on_editor_state_callback);
                self.request_clipboard = Some(request_clipboard);
                self.set_clipboard = Some(set_clipboard);
            }
            Msg::SetBlueprint(blueprint) => {
                self.active_buffer = blueprint.into();
            }
            Msg::SetEditorState(editor_state) => {
                self.editor_state = editor_state;
            }
            Msg::SetClipboard(blueprint) => {
                let buffer: Buffer = blueprint.into();
                self.mode = Mode::Visual;
                self.mouse_follow_buffer = Some(buffer.clone());
                self.editor_state.registers.insert("*".to_owned(), buffer);
            }
            Msg::Render(time) => {
                self.draw(time);
            }
            Msg::SetFocus(focused) => {
                if !focused && !matches!(self.mode, Mode::Execute(_)) {
                    self.mode = Mode::Visual;
                }
            }
            Msg::SetModule((root, concrete_module)) => {
                if let Some(concrete_module) = concrete_module {
                    self.active_buffer.modules.insert(root, concrete_module);
                } else {
                    self.active_buffer.modules.remove(&root);
                }

                self.notify_js_on_change();
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

        let mut modules = self.active_buffer.modules.clone();

        if let Some(mouse_follow_buffer) = &self.mouse_follow_buffer {
            for (mut root, mut module) in mouse_follow_buffer.modules.clone() {
                root.0 += self.input_state.cell.0;
                module.set_root(root);
                modules.insert(root, module);
            }
        }

        let modules_html = modules
            .into_iter()
            .map(|(_, module)| match module {
                ConcreteModule::Value(value) => {
                    html! {
                        <ValueComponent
                            key={u64::from(value.get_root())}
                            {value}
                            camera={self.editor_state.camera.clone()}
                            update_self={ctx.link().callback(Msg::SetModule)}
                            edit_mode={matches!(self.mode, Mode::ModuleEdit(..))}
                        />
                    }
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
                    Mode::Label(..) => html!(<span style="color: yellow;">{"Label"}</span>),
                    Mode::ModuleEdit(None) => html!(<span style="color: purple;">{"Module [None]"}</span>),
                    Mode::ModuleEdit(Some(module)) => html! {
                        <span style="color: purple;">
                        {
                            format!("Module [{}]", match module {
                                // ConcreteModule::Clock(_) => "Clock",
                                ConcreteModule::Value(_) => "Value",
                            })
                        }
                        </span>
                    },
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
                                    Mode::Execute(..) | Mode::ModuleEdit(None) => "default",
                                    Mode::Label(..) | Mode::ModuleEdit(Some(..)) => "copy",
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
                            if self.editor_state.selection.is_zero() {
                                "None".to_owned()
                            } else {
                                format!(
                                    "{} -> {}",
                                    self.editor_state.selection.lower_left.0,
                                    self.editor_state.selection.upper_right.0
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
                    {
                        if matches!(self.mode, Mode::ModuleEdit(..))
                           && self.mouse_follow_buffer.is_some() {
                            html!(<span class="lp-no-pointer-events">{modules_html}</span>)
                        } else {
                            modules_html
                        }
                    }
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
            .expect("Failed to construct DomIntervalEvents"),
        );

        let window = window().unwrap();
        let document = document();

        let link = ctx.link().clone();
        let key_down = EventListener::new(&document, "keydown", move |event| {
            let event = event
                .dyn_ref::<web_sys::KeyboardEvent>()
                .expect("Failed to cast keydown event");
            link.send_message(Msg::RawInput(RawInput::KeyDown(event.clone())));
        });

        let link = ctx.link().clone();
        let key_up = EventListener::new(&document, "keyup", move |event| {
            let event = event
                .dyn_ref::<web_sys::KeyboardEvent>()
                .expect("Failed to cast keyup event.");
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
