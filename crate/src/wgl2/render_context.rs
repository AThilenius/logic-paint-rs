use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    session::Session,
    wgl2::{CellProgram, QuadVao, SetUniformType, Texture},
};

use crate::coords::ChunkCoord;

pub struct RenderContext {
    program: CellProgram,
    render_chunks: HashMap<ChunkCoord, RenderChunk>,
    gl: WebGl2RenderingContext,
    empty_texture: Texture,
}

struct RenderChunk {
    cell_texture: Texture,
    mask_texture: Texture,
    vao: QuadVao,
}

impl RenderContext {
    pub fn new(canvas: HtmlCanvasElement) -> Result<RenderContext, JsValue> {
        let options = js_sys::Object::new();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let program = CellProgram::compile(&gl)?;

        let empty_texture = Texture::new_chunk_texture(&gl)?;

        Ok(Self {
            program,
            render_chunks: HashMap::new(),
            gl,
            empty_texture,
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
                let cell_texture = Texture::new_chunk_texture(&self.gl)?;
                let mask_texture = Texture::new_chunk_texture(&self.gl)?;
                self.render_chunks.insert(
                    chunk_coord.clone(),
                    RenderChunk {
                        cell_texture,
                        mask_texture,
                        vao,
                    },
                );
                self.render_chunks.get_mut(&chunk_coord).unwrap()
            };

            // Update and bind the cell texture.
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

            if let Some(buffer_chunk) = session.active_buffer.get_chunk(chunk_coord) {
                render_chunk
                    .cell_texture
                    .set_pixels(&buffer_chunk.cells[..])?;

                // Bind the render chunk's texture as it's non-empty.
                render_chunk.cell_texture.bind();
            } else {
                // Bind the empty texture.
                self.empty_texture.bind();
            }

            // Update and bind the mask texture.
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);

            if let Some(mask_chunk) = session.active_mask.get_chunk(chunk_coord) {
                render_chunk
                    .mask_texture
                    .set_pixels(&mask_chunk.cells[..])?;

                // Bind the mask chunk's texture as it's non-empty.
                render_chunk.mask_texture.bind();
            } else {
                // Bind the empty texture.
                self.empty_texture.bind();
            }

            render_chunk.vao.bind();
            self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }

        Ok(())
    }
}