use std::{cell::RefCell, rc::Rc};

use glam::{IVec2, Mat4, Vec2, Vec3};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    brush::Brush,
    dom::{DomIntervalTarget, ElementEventTarget, ElementInputEvent},
    log,
    substrate::{Cell, IntegratedCircuit, Metal, Silicon},
    unwrap_or_log_and_return,
    wgl2::{Camera, CellChunkTexture, CellProgram, QuadVao, SetUniformType},
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
    cell_chunk_textures: Vec<CellChunkTexture>,
    needs_recompile: bool,
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
        ic.set_cell(
            IVec2::ZERO,
            Cell {
                metal: Metal::IO {
                    dirs: Default::default(),
                },
                si: Silicon::None,
            },
        );

        let viewport = Rc::new(RefCell::new(Self {
            camera: Default::default(),
            ic,
            brush: Brush::new(),
            canvas,
            ctx,
            cell_program: program,
            quad_vao: vao,
            cell_chunk_textures: vec![],
            needs_recompile: false,
        }));

        Ok(viewport)
    }
}

impl DomIntervalTarget for Viewport {
    fn animation_frame(&mut self) {
        let (w, h) = (
            self.canvas.client_width() as u32,
            self.canvas.client_height() as u32,
        );
        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
            self.ctx.viewport(0, 0, w as i32, h as i32);
        }

        self.needs_recompile = self.brush.commit_changes(&mut self.ic);

        self.ctx.clear_color(0.2, 0.2, 0.2, 1.0);
        self.ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.cell_program.use_program(&self.ctx);
        self.quad_vao.bind(&self.ctx);

        self.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            Vec2::new(w as f32, h as f32),
        );
        self.cell_program
            .view_proj
            .set(&self.ctx, self.camera.get_view_proj_matrix());

        // Render visible chunks...
        let visible_chunks = self.camera.get_visible_substrate_chunk_locs();

        for (i, chunk_loc) in visible_chunks.iter().enumerate() {
            if i >= self.cell_chunk_textures.len() {
                self.cell_chunk_textures
                    .push(unwrap_or_log_and_return!(CellChunkTexture::new(&self.ctx)));
            }

            let chunk_texture = &mut self.cell_chunk_textures[i];
            unwrap_or_log_and_return!(chunk_texture.rasterize_ic_chunk(
                &self.ctx,
                &self.ic,
                &self.brush.cell_overrides,
                chunk_loc
            ));

            // Bind and draw
            self.cell_program.model.set(
                &self.ctx,
                Mat4::from_translation(Vec3::new(chunk_loc.x as f32, chunk_loc.y as f32, 0.0)),
            );
            chunk_texture.bind(&mut self.ctx);
            self.ctx
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }
    }

    fn simulate_step(&mut self) -> bool {
        if self.needs_recompile {
            self.ic.compile();
        }

        false
    }
}

impl ElementEventTarget for Viewport {
    fn on_input_event(&mut self, event: ElementInputEvent) {
        self.camera.handle_mouse_event(event.clone());
        self.brush.handle_input_event(&self.camera, &self.ic, event);
    }
}

impl Drop for Viewport {
    fn drop(&mut self) {
        log!("Canvas viewport dropped");
    }
}