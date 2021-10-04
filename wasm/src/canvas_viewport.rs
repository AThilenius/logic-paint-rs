use glam::Mat4;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

use crate::substrate_render_chunk::SubstrateRenderChunk;
use crate::wgl2::{Camera, CellProgram, QuadVao, SetUniformType};

/// Manages a HTML Canvas element, rendering a viewport of a Substrate.
pub struct CanvasViewport {
    pub camera: Camera,
    ctx: WebGl2RenderingContext,
    cell_program: CellProgram,
    quad_vao: QuadVao,
    test: SubstrateRenderChunk,
}

impl CanvasViewport {
    pub fn from_canvas(canvas: web_sys::HtmlCanvasElement) -> Result<CanvasViewport, JsValue> {
        let (w, h) = (canvas.client_width() as u32, canvas.client_height() as u32);
        canvas.set_width(w);
        canvas.set_height(h);

        let ctx = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = CellProgram::compile(&ctx)?;
        program.use_program(&ctx);

        let camera = Camera::default();

        // Set program defaults
        program.view_proj.set(&ctx, camera.get_view_proj_matrix());
        program.model.set(&ctx, Mat4::IDENTITY);

        let vao = QuadVao::new(&ctx, &program.program)?;
        let test = SubstrateRenderChunk::new(&ctx)?;

        Ok(Self {
            camera,
            ctx,
            cell_program: program,
            quad_vao: vao,
            test,
        })
    }

    pub fn draw(&self) {
        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.cell_program.use_program(&self.ctx);
        self.quad_vao.bind(&self.ctx);

        self.cell_program
            .view_proj
            .set(&self.ctx, self.camera.get_view_proj_matrix());

        self.ctx
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    }
}
