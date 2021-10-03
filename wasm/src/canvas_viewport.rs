use glam::Mat4;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlVertexArrayObject};

use crate::substrate_render_chunk::SubstrateRenderChunk;
use crate::wgl2::{CellProgram, QuadVao};

/// Manages a HTML Canvas element, rendering a viewport of a Substrate.
pub struct CanvasViewport {
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

        program.set_view_proj(&ctx, w, h, Mat4::IDENTITY);
        program.set_model(&ctx, Mat4::IDENTITY);

        let vao = QuadVao::new(&ctx, &program.program)?;

        draw(&ctx);

        Ok(Self {
            cell_program: program,
            quad_vao: vao,
            test: SubstrateRenderChunk::new(&ctx)?,
        })
    }
}

fn draw(context: &WebGl2RenderingContext) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
}
