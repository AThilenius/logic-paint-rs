use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::{IVec2, Vec2};
use miniz_oxide::deflate::compress_to_vec;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    brush::Brush,
    dom::{DomIntervalTarget, ElementEventTarget, ElementInputEvent},
    log, result_or_log_and_return,
    substrate::{serialize_ic, IntegratedCircuit, PinModule, SimIcState},
    wgl2::{Camera, CellProgram, CellRenderChunk, SetUniformType},
};

/// Manages a HTML Canvas element, rendering a viewport of a Substrate. This struct is always stored
/// in a Rc because it's accessed from JS callbacks. To free the struct, the callbacks must be
/// un-registered with `drop_callbacks()` before dropping other Rc instances.
pub struct Viewport {
    pub camera: Camera,
    pub ic: IntegratedCircuit,
    pub brush: Brush,
    canvas: HtmlCanvasElement,
    cell_program: CellProgram,
    cell_render_chunks: HashMap<IVec2, CellRenderChunk>,
    ctx: WebGl2RenderingContext,
    deferred_mouse_event: Option<ElementInputEvent>,
    deferred_buttons_down: (bool, bool),
    sim_state: Option<SimIcState>,
}

impl Viewport {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Rc<RefCell<Viewport>>, JsValue> {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"desynchronized".into(), &JsValue::TRUE)?;
        js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &JsValue::TRUE)?;
        let ctx = canvas
            // .get_context("webgl2")?
            .get_context_with_context_options("webgl2", &options)?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = CellProgram::compile(&ctx)?;
        program.use_program(&ctx);

        let mut ic = IntegratedCircuit::default();

        ic.add_pin_module(PinModule::ConstVal {
            cell_loc: IVec2::new(-4, 16),
            high: true,
        });
        ic.add_pin_module(PinModule::Clock {
            cell_loc: IVec2::new(-4, 14),
            interval: 30,
            name: "CLK".to_string(),
            starts_high: false,
        });

        let viewport = Rc::new(RefCell::new(Self {
            camera: Default::default(),
            ic,
            brush: Brush::new(),
            canvas,
            cell_program: program,
            cell_render_chunks: Default::default(),
            ctx,
            deferred_mouse_event: None,
            deferred_buttons_down: (false, false),
            sim_state: None,
        }));

        Ok(viewport)
    }

    pub fn set_ic(&mut self, ic: IntegratedCircuit) {
        self.ic = ic;
        for (_, render_chunk) in self.cell_render_chunks.iter_mut() {
            // TODO: This needs to be stored in self.
            render_chunk.layout_dirty = true;
            render_chunk.trace_cache_dirty = true;
        }
    }
}

impl DomIntervalTarget for Viewport {
    fn animation_frame(&mut self, time: f64) {
        // DEV
        if let Some(sim_ic_state) = self.sim_state.take() {
            self.sim_state = Some(self.ic.step_simulation_state(sim_ic_state));
        }
        // DEV

        // Maintain HTML Canvas size and context viewport.
        let (w, h) = (
            self.canvas.client_width() as u32,
            self.canvas.client_height() as u32,
        );
        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
            self.ctx.viewport(0, 0, w as i32, h as i32);
        }

        // Pass on any deferred mouse events to the Brush.
        if let Some(event) = self.deferred_mouse_event.take() {
            self.brush.handle_input_event(&self.camera, &self.ic, event);
        }

        // Update camera uniform.
        self.cell_program.use_program(&self.ctx);
        self.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );
        self.cell_program
            .view_proj
            .set(&self.ctx, self.camera.get_view_proj_matrix());
        self.cell_program.time.set(&self.ctx, time as f32);

        // Render visible chunks.
        let visible_chunks = self.camera.get_visible_substrate_chunk_locs();
        let dirty_chunks = self.brush.get_effected_chunks();
        let should_rebuild_trace_caches = self.brush.commit_changes(&mut self.ic);

        // TODO: This really shouldn't go here...
        if should_rebuild_trace_caches {
            let compressed_bytes = compress_to_vec(&serialize_ic(&self.ic), 6);
            let compressed_b64 = base64::encode(compressed_bytes);
            web_sys::window()
                .unwrap()
                .local_storage()
                .unwrap()
                .unwrap()
                .set_item(&"logic-paint-ic", &compressed_b64)
                .expect("Failed to save IC");
        }

        for chunk_loc in visible_chunks {
            let cell_render_chunk = if let Some(crc) = self.cell_render_chunks.get_mut(&chunk_loc) {
                crc
            } else {
                self.cell_render_chunks.insert(
                    chunk_loc.clone(),
                    result_or_log_and_return!(CellRenderChunk::new(
                        &self.ctx,
                        &self.cell_program,
                        &chunk_loc
                    )),
                );
                self.cell_render_chunks.get_mut(&chunk_loc).unwrap()
            };

            let layout_dirty = dirty_chunks.contains(&chunk_loc);
            // Or-equal because it could be dirty from last loop.
            cell_render_chunk.layout_dirty |= layout_dirty;
            cell_render_chunk.trace_cache_dirty = should_rebuild_trace_caches;

            result_or_log_and_return!(cell_render_chunk.draw(
                &self.ic,
                &self.brush.cell_overrides,
                &self.sim_state,
            ));

            // Set dirty a second time. This covers the case where a chunk was being drawn in but
            // isn't any more.
            cell_render_chunk.layout_dirty = layout_dirty;
        }
    }

    fn simulate_step(&mut self) -> bool {
        false
    }
}

impl ElementEventTarget for Viewport {
    fn on_input_event(&mut self, event: ElementInputEvent) {
        self.camera.handle_mouse_event(event.clone());

        match &event {
            ElementInputEvent::MouseDown(e)
            | ElementInputEvent::MouseUp(e)
            | ElementInputEvent::MouseMove(e) => {
                // Mouse events to the Brush are only sent in the event handler if the button state changed
                // from the last event to now. Otherwise the latest event is simply deferred till render
                // time.
                let pressed = (e.buttons() & 1 != 0, e.buttons() & 2 != 0);
                if pressed != self.deferred_buttons_down {
                    self.brush
                        .handle_input_event(&self.camera, &self.ic, event.clone());
                    self.deferred_buttons_down = pressed;
                } else {
                    self.deferred_mouse_event = Some(event.clone());
                }
            }
            _ => {
                // All other events are sent right now.
                self.brush
                    .handle_input_event(&self.camera, &self.ic, event.clone());
            }
        };

        match event {
            ElementInputEvent::KeyPressed(event) if event.code() == "KeyC" => {
                if let Some(_) = self.sim_state {
                    self.sim_state = None;
                } else {
                    self.sim_state = Some(self.ic.build_new_sim_state());
                }
            }
            _ => {}
        }
    }
}

impl Drop for Viewport {
    fn drop(&mut self) {
        log!("Canvas viewport dropped");
    }
}
