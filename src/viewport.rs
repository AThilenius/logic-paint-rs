use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    coords::ChunkCoord,
    substrate::{buffer::Buffer, mask::Mask},
    utils::Selection,
    wgl2::{Camera, CellProgram, QuadVao, SetUniformType, Texture},
};

/// Represents only the presentation state of a on or off screen viewport for rendering.
#[wasm_bindgen(getter_with_clone)]
pub struct Viewport {
    pub selection: Selection,
    pub camera: Camera,
    pub buffer: Buffer,
    pub mask: Mask,
    pub time: f64,

    program: CellProgram,
    render_chunks: HashMap<ChunkCoord, RenderChunk>,
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    empty_texture: Texture,
}

struct RenderChunk {
    cell_texture: Texture,
    mask_texture: Texture,
    vao: QuadVao,
}

#[wasm_bindgen]
impl Viewport {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let options = js_sys::Object::new();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .expect("get webgl2 context from canvas")
            .expect("webgl2 context to be non-null")
            .dyn_into::<WebGl2RenderingContext>()
            .expect("cast get_context_with_context_options into WebGL2RenderingContext");

        let program = CellProgram::compile(&gl).expect("glsl programs to compile");
        let empty_texture = Texture::new_chunk_texture(&gl).expect("webgl textures to create");

        Self {
            selection: Default::default(),
            camera: Default::default(),
            buffer: Default::default(),
            mask: Default::default(),
            time: Default::default(),
            render_chunks: Default::default(),
            program,
            canvas,
            gl,
            empty_texture,
        }
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        self.time = web_sys::window()
            .expect("should have a window in this context")
            .performance()
            .expect("performance should be available")
            .now();

        // Maintain HTML Canvas size and context viewport.
        let w = self.canvas.client_width() as u32;
        let h = self.canvas.client_height() as u32;

        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }

        self.camera.update((w as f32, h as f32).into());

        self.gl
            .viewport(0, 0, self.camera.size.x as i32, self.camera.size.y as i32);

        // Update per-draw uniforms
        self.program.use_program(&self.gl);
        self.program
            .view_proj
            .set(&self.gl, self.camera.get_view_proj_matrix());
        self.program.time.set(&self.gl, self.time as f32);
        self.program
            .cell_select_ll
            .set(&self.gl, self.selection.lower_left.0);
        self.program
            .cell_select_ur
            .set(&self.gl, self.selection.upper_right.0);

        // Get chunks visible to the camera.
        let visible_chunks = self.camera.get_visible_chunk_coords();

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

            if let Some(buffer_chunk) = self.buffer.chunks.get(&chunk_coord) {
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

            if let Some(mask_chunk) = self.mask.get_chunk(chunk_coord) {
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
}
