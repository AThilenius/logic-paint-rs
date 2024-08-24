use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    utils::Selection,
    viewport::{buffer::Buffer, mask::Mask},
    wgl2::{Camera, CellProgram, QuadVao, SetUniformType, Texture},
};

use crate::coords::{CellCoord, ChunkCoord};

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

    pub fn draw(
        &mut self,
        time: f64,
        buffer: &Buffer,
        selection: &Selection,
        mask: Option<&Mask>,
        camera: &Camera,
    ) -> Result<(), JsValue> {
        self.gl
            .viewport(0, 0, camera.size.x as i32, camera.size.y as i32);

        // Update per-draw uniforms
        self.program.use_program(&self.gl);
        self.program
            .view_proj
            .set(&self.gl, camera.get_view_proj_matrix());
        self.program.time.set(&self.gl, time as f32);
        self.program
            .cell_select_ll
            .set(&self.gl, selection.lower_left.0);
        self.program
            .cell_select_ur
            .set(&self.gl, selection.upper_right.0);

        // Get chunks visible to the camera.
        let visible_chunks = camera.get_visible_chunk_coords();

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

            if let Some(buffer_chunk) = buffer.chunks.get(&chunk_coord) {
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

            if let Some(mask_chunk) = mask.and_then(|m| m.get_chunk(chunk_coord)) {
                render_chunk
                    .mask_texture
                    .set_pixels(&mask_chunk.cells[..])?;

                // Bind the mask chunk's texture as it's non-empty.
                render_chunk.mask_texture.bind();
            } else {
                // Bind the empty texture.
                self.empty_texture.bind();
            }

            // Update the per-chunk program uniforms
            self.program
                .chunk_start_cell_offset
                .set(&self.gl, chunk_coord.first_cell_coord().0);

            // Bind the VAO and draw
            render_chunk.vao.bind();
            self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }

        Ok(())
    }

    pub fn set_cursor_coord(&self, coord: CellCoord) {
        self.program.cursor_coord.set(&self.gl, coord.0);
    }
}
