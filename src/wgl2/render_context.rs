use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    log,
    session::Session,
    wgl2::{CellProgram, QuadVao, SetUniformType, Texture},
};

use crate::coords::ChunkCoord;

pub struct RenderContext {
    program: CellProgram,
    render_chunks: HashMap<ChunkCoord, RenderChunk>,
    gl: WebGl2RenderingContext,
}

struct RenderChunk {
    buffer_generation: usize,
    texture: Texture,
    vao: QuadVao,
}

impl RenderContext {
    pub fn new(canvas: HtmlCanvasElement) -> Result<RenderContext, JsValue> {
        // Desync buffer, see: https://developers.google.com/web/updates/2019/05/desynchronized
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"desynchronized".into(), &JsValue::TRUE)?;
        js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &JsValue::TRUE)?;

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = CellProgram::compile(&gl)?;

        Ok(Self {
            program,
            render_chunks: HashMap::new(),
            gl,
        })
    }

    pub fn draw(&mut self, time: f64, session: &Session) -> Result<(), JsValue> {
        self.gl.viewport(
            0,
            0,
            session.camera.size.x as i32,
            session.camera.size.y as i32,
        );

        // Update camera uniform.
        self.program.use_program(&self.gl);
        self.program
            .view_proj
            .set(&self.gl, session.camera.get_view_proj_matrix());
        self.program.time.set(&self.gl, time as f32);

        // Get chunks visible to the camera.
        let visible_chunks = session.camera.get_visible_chunk_coords();

        // Drop RenderChunks that aren't visible any more.
        self.render_chunks.retain(|c, _| visible_chunks.contains(c));

        for chunk_coord in visible_chunks {
            let render_chunk = if let Some(crc) = self.render_chunks.get_mut(&chunk_coord) {
                crc
            } else {
                let vao = QuadVao::new(&self.gl, &self.program, &chunk_coord)?;
                let texture = Texture::new_chunk_texture(&self.gl)?;
                self.render_chunks.insert(
                    chunk_coord.clone(),
                    RenderChunk {
                        buffer_generation: usize::MAX,
                        texture,
                        vao,
                    },
                );
                self.render_chunks.get_mut(&chunk_coord).unwrap()
            };

            // Draw the RenderChunk...
            if let Some(buffer_chunk) = session.active_buffer.get_chunk(chunk_coord) {
                if render_chunk.buffer_generation != buffer_chunk.generation {
                    render_chunk.buffer_generation = buffer_chunk.generation;
                    render_chunk.texture.set_pixels(&buffer_chunk.cells[..])?;
                }
            }

            render_chunk.texture.bind();
            render_chunk.vao.bind();
            self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }

        Ok(())
    }
}
