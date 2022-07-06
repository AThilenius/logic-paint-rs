use glam::{IVec2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement};
use yew::prelude::*;

use crate::{
    blueprint::Blueprint,
    brush::{draw_metal, draw_si, draw_via},
    buffer::Buffer,
    buffer_mask::BufferMask,
    dom::{DomIntervalHooks, ModuleMount, RawInput},
    execution_context::ExecutionContext,
    input::InputState,
    upc::{Metal, NormalizedCell, Silicon},
    utils::Selection,
    wgl2::{Camera, RenderContext},
};

pub struct Viewport {
    pub active_buffer: Buffer,
    pub ephemeral_buffer: Option<Buffer>,
    pub execution_context: Option<ExecutionContext>,
    pub selection: Option<Selection>,
    pub selection_mask: BufferMask,
    pub ephemeral_selection: Option<Selection>,
    pub camera: Camera,
    pub time: f64,
    pub input_state: InputState,
    mode: Mode,
    canvas: NodeRef,
    render_context: Option<RenderContext>,
    dom_events: Option<DomIntervalHooks>,
    on_edit_callback: Option<js_sys::Function>,
}

pub enum Msg {
    None,
    SetOnEditCallback(js_sys::Function),
    SetBlueprintPartial(Blueprint),
    PasteBlueprint(Blueprint),
    RawInput(RawInput),
    Render(f64),
    StartStopSim,
    SetFocus(bool),
}

#[derive(PartialEq, Eq)]
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
}

impl Viewport {
    pub fn copy(&self) -> Option<Blueprint> {
        if let Some(selection) = self
            .selection
            .as_ref()
            .or(self.ephemeral_selection.as_ref())
        {
            let buffer = self.active_buffer.clone_selection(&selection);
            let mut blueprint = Blueprint::from(&buffer);
            blueprint.root_offset = Some(-self.input_state.mouse_input.cell.0);
            Some(blueprint)
        } else {
            None
        }
    }

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

        // Update the selection mask
        self.selection_mask = Default::default();
        let merged_selection = {
            if let Some(selection) = &self.selection {
                if let Some(ephemeral_selection) = &self.ephemeral_selection {
                    if self.input_state.keyboard_input.ctrl {
                        Some(selection.difference(ephemeral_selection))
                    } else {
                        Some(selection.union(ephemeral_selection))
                    }
                } else {
                    Some(selection.clone())
                }
            } else {
                self.ephemeral_selection.clone()
            }
        };
        if let Some(selection) = merged_selection {
            for (chunk_coord, cell_coords) in selection.group_changes_by_chunk() {
                let chunk = self.selection_mask.get_or_create_chunk_mut(chunk_coord);

                for cell_coord in cell_coords {
                    chunk.set_cell_active(cell_coord);
                }
            }
        }

        if let Some(render_context) = &mut self.render_context {
            let buffer = self
                .ephemeral_buffer
                .as_ref()
                .unwrap_or(&self.active_buffer);

            let mask = self
                .execution_context
                .as_ref()
                .map(|c| &c.buffer_mask)
                .unwrap_or(&self.selection_mask);

            render_context
                .draw(time, buffer, mask, &self.camera)
                .unwrap_throw();
        }
    }

    fn dispatch_input_state(&mut self) {
        if self.camera.handle_input(&self.input_state) {
            return;
        }

        // Keybinds: Esc => Visual, D => PaintSi, F => PaintMetallic
        if self.input_state.keyboard_input.keydown.contains("Escape") {
            self.mode = Mode::Visual;
            self.selection = None;
        } else if self.input_state.keyboard_input.keydown.contains("KeyD") {
            self.mode = Mode::PaintSi;
        } else if self.input_state.keyboard_input.keydown.contains("KeyF") {
            self.mode = Mode::PaintMetallic;
        }

        match self.mode {
            Mode::Visual => {
                self.dispatch_visual_input_state();
            }
            Mode::PaintMetallic | Mode::PaintSi => {
                self.dispatch_paint_input_state();
            }
        }
    }

    fn dispatch_visual_input_state(&mut self) {
        let primary = self.input_state.mouse_input.primary;
        let secondary = self.input_state.mouse_input.secondary;

        // Neither button is clicked, see if we need to commit the ephemeral selection to the base
        // selection.
        if !primary && !secondary {
            if let Some(ephemeral_selection) = self.ephemeral_selection.take() {
                if self.input_state.previous_keyboard_input.ctrl {
                    // Subtract it if (and only if) there is a base selection.
                    if let Some(selection) = &self.selection {
                        self.selection = Some(selection.difference(&ephemeral_selection));
                    }
                } else {
                    // Add it to the base selection (or become the base if there is none).
                    self.selection = self
                        .selection
                        .take()
                        .map(|s| s.union(&ephemeral_selection))
                        .or(Some(ephemeral_selection));
                }
            }

            return;
        }

        // If shift isn't down, we can indiscriminately clear out the base selection.
        if !self.input_state.keyboard_input.shift {
            self.selection = None;
        }

        // At this point we don't care if it's primary or secondary, we are drawing into the
        // ephemeral buffer regardless.
        if let Some(drag) = self.input_state.mouse_input.drag {
            self.ephemeral_selection = Some(Selection::from_rectangle(
                drag.start,
                self.input_state.mouse_input.cell,
            ));
        }
    }

    fn dispatch_paint_input_state(&mut self) {
        // TODO: Early-return if the mouse didn't move a cell.
        let primary = self.input_state.mouse_input.primary;
        let secondary = self.input_state.mouse_input.secondary;

        // If neither button is clicked
        if !primary && !secondary {
            // Commit the ephemeral buffer if we have one.
            if let Some(buffer) = self.ephemeral_buffer.take() {
                self.active_buffer = buffer;

                // Notify JS.
                if let Some(on_edit_callback) = self.on_edit_callback.as_ref() {
                    let json = serde_json::to_string_pretty(&Blueprint::from(&self.active_buffer))
                        .unwrap_throw();
                    let js_str = JsValue::from(&json);
                    let _ = on_edit_callback.call1(&JsValue::null(), &js_str);
                }
            }

            return;
        }

        // Painting generates a totally new Buffer (cloned from active) each time.
        let mut buffer = self.active_buffer.clone();

        let path = self.input_state.get_impulse_drag_path();

        // If Ctrl is held down, then we are clearing. The logic for clearing is totally different
        // from painting, so we handle it separately. These of we. The preverbal we. It's just me.
        if self.input_state.keyboard_input.ctrl {
            for cell_coord in path {
                let mut normalized: NormalizedCell = buffer.get_cell(cell_coord.clone()).into();

                match self.mode {
                    Mode::PaintMetallic => normalized.metal = Metal::None,
                    Mode::PaintSi => normalized.si = Silicon::None,
                    _ => {}
                }

                buffer.set_cell(cell_coord, normalized.into());
            }
        } else {
            // Input modes are much, much more complicated. That logic is delegated to it's own file
            // because they are so stupid-complicated.
            let mut from = None;

            for cell_coord in path {
                match self.mode {
                    Mode::PaintMetallic => {
                        // Primary paints metal, secondary places a Via (only once).
                        if primary {
                            draw_metal(&mut buffer, from, cell_coord);
                        } else if secondary {
                            draw_via(&mut buffer, from, cell_coord);
                        }
                    }
                    Mode::PaintSi => {
                        // Primary paints N-type, secondary paints P-type.
                        if primary {
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
}

impl Component for Viewport {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active_buffer: Buffer::default(),
            ephemeral_buffer: None,
            execution_context: None,
            selection: None,
            ephemeral_selection: None,
            selection_mask: BufferMask::default(),
            camera: Camera::new(),
            time: 0.0,
            input_state: InputState::new(),
            mode: Mode::Visual,
            canvas: NodeRef::default(),
            render_context: None,
            dom_events: None,
            on_edit_callback: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RawInput(raw_input) => {
                self.input_state.handle_raw_input(&self.camera, &raw_input);
                self.dispatch_input_state();
            }
            Msg::SetOnEditCallback(callback) => {
                self.on_edit_callback = Some(callback);
            }
            Msg::SetBlueprintPartial(blueprint) => {
                if let Some(new_buffer) = blueprint.into_buffer_from_partial(&self.active_buffer) {
                    self.active_buffer = new_buffer;
                }
            }
            Msg::PasteBlueprint(blueprint) => {
                if let Some(new_buffer) = blueprint.into_buffer_from_partial(&Buffer::default()) {
                    self.active_buffer.paste_buffer_offset_by(
                        blueprint.root_offset.unwrap_or(IVec2::ZERO),
                        &new_buffer,
                    );
                }
            }
            Msg::Render(time) => {
                // Update modules.
                for (_, anchored_module) in self.active_buffer.anchored_modules.iter_mut() {
                    anchored_module.module.borrow_mut().tick(time);
                }

                // Run the sim loop once.
                if let Some(execution_context) = &mut self.execution_context {
                    execution_context.step();
                    execution_context.update_buffer_mask();
                }

                self.draw(time);
            }
            Msg::StartStopSim => {
                if self.execution_context.is_some() {
                    self.execution_context = None;
                } else {
                    self.execution_context =
                        Some(ExecutionContext::compile_from_buffer(&self.active_buffer));
                }
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
        let onkeydown = ctx.link().callback(|e| Msg::RawInput(RawInput::KeyDown(e)));
        let onkeyup = ctx.link().callback(|e| Msg::RawInput(RawInput::KeyUp(e)));
        let oncontextmenu = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            Msg::None
        });
        let onfocus = ctx.link().callback(|_| Msg::SetFocus(true));
        let onblur = ctx.link().callback(|_| Msg::SetFocus(false));

        let modules_html = self
            .active_buffer
            .anchored_modules
            .values()
            .map(|m| {
                html! {
                    <ModuleMount camera={self.camera.clone()} anchor={m.anchor} module_html={m.html.clone()} />
                }
            })
            .collect::<Html>();

        html! {
            <div
                class="lp-viewport"
                {onkeydown}
                {onkeyup}
                {onfocus}
                {onblur}
                tabindex={0}
            >
                <canvas
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {oncontextmenu}
                    {onwheel}
                    ref={self.canvas.clone()}
                    style={format!("
                        cursor: {};
                    ", match self.mode {
                        Mode::Visual => "cell",
                        Mode::PaintMetallic | Mode::PaintSi => "crosshair"
                    })}
                />
                <span
                    style={if self.mode != Mode::Visual { "pointer-events: none;"} else {""}}
                >
                    <div class="lp-bottom-bar">
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
                            if self.mode == Mode::PaintSi { "magenta" } else { "gray" })}
                        >
                            {"Paint Silicon"}
                        </button>
                        <button
                            style={format!("
                                height: 40px;
                                background: {};
                            ",
                            if self.mode == Mode::PaintMetallic { "magenta" } else { "gray" })}
                        >
                            {"Paint Metallic"}
                        </button>
                    </div>
                    <span>{ modules_html }</span>
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
    }
}
