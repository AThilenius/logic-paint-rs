use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::{IVec2, Mat4, Vec2, Vec3};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    brush::Brush,
    dom::{DomIntervalTarget, ElementEventTarget, ElementInputEvent},
    log, result_or_log_and_return,
    substrate::{IntegratedCircuit, PinModule, SimIcState},
    wgl2::{Camera, CellProgram, CellRenderChunk, QuadVao, SetUniformType},
};

/// Manages a HTML Canvas element, rendering a viewport of a Substrate. This struct is always stored
/// in a Rc because it's accessed from JS callbacks. To free the struct, the callbacks must be
/// un-registered with `drop_callbacks()` before dropping other Rc instances.
pub struct Viewport {
    pub camera: Camera,
    pub ic: IntegratedCircuit,
    pub brush: Brush,
    canvas: HtmlCanvasElement,
    ctx: WebGl2RenderingContext,
    cell_program: CellProgram,
    quad_vao: QuadVao,
    cell_render_chunks: HashMap<IVec2, CellRenderChunk>,
    sim_state: Option<SimIcState>,
}

impl Viewport {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Rc<RefCell<Viewport>>, JsValue> {
        let ctx = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = CellProgram::compile(&ctx)?;
        program.use_program(&ctx);

        let vao = QuadVao::new(&ctx, &program.program)?;
        let mut ic = IntegratedCircuit::default();

        ic.add_pin_module(PinModule::Clock {
            cell_loc: IVec2::ZERO,
            interval: 1,
            name: "CLK".to_string(),
            starts_high: false,
        });

        let viewport = Rc::new(RefCell::new(Self {
            camera: Default::default(),
            ic,
            brush: Brush::new(),
            canvas,
            ctx,
            cell_program: program,
            quad_vao: vao,
            cell_render_chunks: Default::default(),
            sim_state: None,
        }));

        Ok(viewport)
    }
}

impl DomIntervalTarget for Viewport {
    fn animation_frame(&mut self, time: f64) {
        // DEV
        if let Some(sim_ic_state) = &self.sim_state {
            self.sim_state = Some(self.ic.step_simulation_state(&sim_ic_state));
        }
        // DEV

        let (w, h) = (
            self.canvas.client_width() as u32,
            self.canvas.client_height() as u32,
        );
        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
            self.ctx.viewport(0, 0, w as i32, h as i32);
        }

        let rebuild_trace_caches = self.brush.commit_changes(&mut self.ic);
        let rebuild_layout = self.brush.cell_overrides.len() > 0;

        self.cell_program.use_program(&self.ctx);
        self.quad_vao.bind(&self.ctx);

        // Update uniforms
        self.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );
        self.cell_program
            .view_proj
            .set(&self.ctx, self.camera.get_view_proj_matrix());
        self.cell_program.time.set(&self.ctx, time as f32);

        // Render visible chunks...
        let visible_chunks = self.camera.get_visible_substrate_chunk_locs();

        for chunk_loc in visible_chunks {
            let cell_render_chunk = if let Some(crc) = self.cell_render_chunks.get_mut(&chunk_loc) {
                crc
            } else {
                self.cell_render_chunks.insert(
                    chunk_loc.clone(),
                    result_or_log_and_return!(CellRenderChunk::new(&self.ctx)),
                );
                self.cell_render_chunks.get_mut(&chunk_loc).unwrap()
            };

            if rebuild_layout {
                result_or_log_and_return!(cell_render_chunk.reset_and_update_layout(
                    &self.ctx,
                    &self.ic,
                    &self.brush.cell_overrides,
                    &chunk_loc,
                ));
            } else if rebuild_trace_caches {
                cell_render_chunk.rebuild_trace_state_lookaside_cache(&self.ic, &chunk_loc);
            }

            if let Some(sim_ic_state) = &self.sim_state {
                result_or_log_and_return!(
                    cell_render_chunk.update_trace_active(&self.ctx, sim_ic_state)
                );
            }

            self.cell_program.model.set(
                &self.ctx,
                Mat4::from_translation(Vec3::new(chunk_loc.x as f32, chunk_loc.y as f32, 0.0)),
            );
            cell_render_chunk.bind(&mut self.ctx);
            self.ctx
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }
    }

    fn simulate_step(&mut self) -> bool {
        false
    }
}

impl ElementEventTarget for Viewport {
    fn on_input_event(&mut self, event: ElementInputEvent) {
        self.camera.handle_mouse_event(event.clone());
        self.brush
            .handle_input_event(&self.camera, &self.ic, event.clone());

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
