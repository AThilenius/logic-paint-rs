use std::{cell::RefCell, rc::Rc};

use glam::Mat4;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    dom::{DomIntervalTarget, ElementEventTarget},
    log,
    substrate_render_chunk::SubstrateRenderChunk,
    wgl2::{Camera, CellProgram, QuadVao, SetUniformType},
};

/// Manages a HTML Canvas element, rendering a viewport of a Substrate. This struct is always stored
/// in a Rc because it's accessed from JS callbacks. To free the struct, the callbacks must be
/// un-registered with `drop_callbacks()` before dropping other Rc instances.
pub struct CanvasViewport {
    pub camera: Camera,
    canvas: HtmlCanvasElement,
    ctx: WebGl2RenderingContext,
    cell_program: CellProgram,
    quad_vao: QuadVao,
    test: SubstrateRenderChunk,
}

impl CanvasViewport {
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<Rc<RefCell<CanvasViewport>>, JsValue> {
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

        let viewport = Rc::new(RefCell::new(Self {
            camera,
            canvas,
            ctx,
            cell_program: program,
            quad_vao: vao,
            test,
        }));

        Ok(viewport)
    }
}

impl DomIntervalTarget for CanvasViewport {
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

        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.cell_program.use_program(&self.ctx);
        self.quad_vao.bind(&self.ctx);

        self.camera.update(
            window().unwrap().device_pixel_ratio() as f32,
            w as f32,
            h as f32,
        );
        self.cell_program
            .view_proj
            .set(&self.ctx, self.camera.get_view_proj_matrix());

        self.ctx
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    }

    fn simulate_step(&mut self) -> bool {
        false
    }
}

impl ElementEventTarget for CanvasViewport {
    fn on_mouse(&mut self, event: crate::dom::ElementMouseEvent) {
        log!("{:#?}", event);
    }
}

impl Drop for CanvasViewport {
    fn drop(&mut self) {
        log!("Canvas viewport dropped");
    }
}
