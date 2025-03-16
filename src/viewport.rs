use std::collections::HashMap;

use glam::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    coords::ChunkCoord,
    editor::Editor,
    wgl2::{Camera, CellProgram, QuadVao, SetUniformType, Texture},
};

/// Represents only the presentation state of an on or off screen viewport for rendering.
#[wasm_bindgen]
pub struct Viewport {
    program: CellProgram,
    render_chunks: HashMap<ChunkCoord, RenderChunk>,
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
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

        Self {
            render_chunks: Default::default(),
            program,
            canvas,
            gl,
        }
    }

    pub fn draw(&mut self, camera: &mut Camera, editor: &Editor) -> Result<(), JsValue> {
        let time: f64 = web_sys::window()
            .expect("should have a window in this context")
            .performance()
            .expect("performance should be available")
            .now()
            / 1000.0;

        // Maintain HTML Canvas size and context viewport.
        let w = self.canvas.client_width() as u32;
        let h = self.canvas.client_height() as u32;

        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }

        camera.update((w as f32, h as f32).into());

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
            .set(&self.gl, editor.selection.lower_left.0);
        self.program
            .cell_select_ur
            .set(&self.gl, editor.selection.upper_right.0);
        self.program.cursor_coord.set(
            &self.gl,
            editor
                .cursor_coord
                .map(|c| c.0)
                .unwrap_or((i32::MIN, i32::MIN).into()),
        );

        // Compute the lower-left and upper-right chunk coords visible to the camera.
        let ll: ChunkCoord = camera
            .project_screen_point_to_cell(Vec2::new(-1.0, camera.size.y + 1.0))
            .into();
        let ur: ChunkCoord = camera
            .project_screen_point_to_cell(Vec2::new(camera.size.x + 1.0, -1.0))
            .into();
        let width = ur.0.x - ll.0.x;
        let height = ur.0.y - ll.0.y;

        // Render the background VAO.
        let vao = QuadVao::new(&self.gl, &self.program, &ll, width.max(height) as u32 + 1)?;
        // Bind empty cell and mask textures
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.program
            .chunk_start_cell_offset
            .set(&self.gl, ll.first_cell_coord().0);

        // Bind the VAO and draw
        vao.bind();
        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        // Then render all non-empty chunks
        for chunk in &editor.buffer.chunks {
            // Frustum cull chunks that aren't visible
            let coord = chunk.chunk_coord;
            if coord.0.x < ll.0.x || coord.0.x > ur.0.x || coord.0.y < ll.0.y || coord.0.y > ur.0.y
            {
                continue;
            }

            let render_chunk = if let Some(crc) = self.render_chunks.get_mut(&coord) {
                crc
            } else {
                let vao = QuadVao::new(&self.gl, &self.program, &coord, 1)?;
                let cell_texture = Texture::new_chunk_texture(&self.gl)?;
                let mask_texture = Texture::new_chunk_texture(&self.gl)?;
                self.render_chunks.insert(
                    coord.clone(),
                    RenderChunk {
                        cell_texture,
                        mask_texture,
                        vao,
                    },
                );
                self.render_chunks.get_mut(&coord).unwrap()
            };

            // Update and bind the cell texture.
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

            // Blit cells
            render_chunk
                .cell_texture
                .set_pixels(&chunk.get_cells()[..])?;

            // Bind the render chunk's texture as it's non-empty.
            render_chunk.cell_texture.bind();

            // Update and bind the mask texture.
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);

            if let Some(mask_chunk) = editor.mask.get_chunk(coord) {
                render_chunk
                    .mask_texture
                    .set_pixels(&mask_chunk.cells[..])?;

                // Bind the mask chunk's texture as it's non-empty.
                render_chunk.mask_texture.bind();
            } else {
                // Bind the empty texture.
                self.gl
                    .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            }

            // Update the per-chunk program uniforms
            self.program
                .chunk_start_cell_offset
                .set(&self.gl, coord.first_cell_coord().0);

            // Bind the VAO and draw
            render_chunk.vao.bind();
            self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
        }

        Ok(())
    }
}
