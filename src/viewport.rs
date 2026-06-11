use std::collections::HashMap;

use glam::{IVec2, Vec2};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    coords::ChunkCoord,
    project::Project,
    render::prep::{collect_nonempty_chunks, prepare_chunk_pixels, ChunkUploadState},
    wgl2::{Camera, CellBgProgram, CellProgram, OverlayUniformState, QuadVao, SetUniformType, Texture},
};

/// Represents only the presentation state of an on or off screen viewport for rendering.
#[wasm_bindgen]
pub struct Viewport {
    program: CellProgram,
    bg_program: CellBgProgram,
    render_chunks: HashMap<ChunkCoord, RenderChunk>,
    background_vao: Option<QuadVao>,
    background_key: Option<(ChunkCoord, u32, u32)>,
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
}

struct RenderChunk {
    cell_texture: Texture,
    vao: QuadVao,
    upload: ChunkUploadState,
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
        let bg_program = CellBgProgram::compile(&gl).expect("background glsl program to compile");

        Self {
            render_chunks: Default::default(),
            background_vao: None,
            background_key: None,
            program,
            bg_program,
            canvas,
            gl,
        }
    }

    pub fn draw(&mut self, camera: &mut Camera, editor: &Project) -> Result<(), JsValue> {
        let time: f64 = web_sys::window()
            .expect("should have a window in this context")
            .performance()
            .expect("performance should be available")
            .now()
            / 1000.0;

        let w = self.canvas.client_width() as u32;
        let h = self.canvas.client_height() as u32;

        if w != self.canvas.width() || h != self.canvas.height() {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
        }

        camera.update((w as f32, h as f32).into());

        self.gl
            .viewport(0, 0, camera.size.x as i32, camera.size.y as i32);

        let ll: ChunkCoord = camera
            .project_screen_point_to_cell(Vec2::new(-1.0, camera.size.y + 1.0))
            .into();
        let ur: ChunkCoord = camera
            .project_screen_point_to_cell(Vec2::new(camera.size.x + 1.0, -1.0))
            .into();
        let width = (ur.0.x - ll.0.x) as u32;
        let height = (ur.0.y - ll.0.y) as u32;
        let width_chunks = width + 1;
        let height_chunks = height + 1;
        let bg_key = (ll, width_chunks, height_chunks);

        if self.background_key != Some(bg_key) {
            self.background_vao = Some(QuadVao::new_rect(
                &self.gl,
                &self.program,
                &ll,
                width_chunks,
                height_chunks,
            )?);
            self.background_key = Some(bg_key);
        }

        let overlay = OverlayUniformState {
            view_proj: camera.get_view_proj_matrix(),
            cell_select_ll: editor.selection.lower_left.0,
            cell_select_ur: editor.selection.upper_right.0,
            cursor_coord: editor
                .cursor_coord
                .map(|c| c.0)
                .unwrap_or(IVec2::new(i32::MIN, i32::MIN)),
        };

        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        self.bg_program.use_program(&self.gl);
        self.bg_program.set_overlay_uniforms(&self.gl, &overlay);
        self.bg_program
            .chunk_start_cell_offset
            .set(&self.gl, ll.first_cell_coord().0);

        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.background_vao.as_ref().expect("background vao").bind();
        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);

        let chunk_index = collect_nonempty_chunks(&editor.buffer);
        self.render_chunks
            .retain(|coord, _| chunk_index.contains_key(coord));

        self.program.use_program(&self.gl);
        self.program.set_overlay_uniforms(&self.gl, &overlay);
        self.program.time.set(&self.gl, time as f32);

        for y in ll.0.y..=ur.0.y {
            for x in ll.0.x..=ur.0.x {
                let coord = ChunkCoord((x, y).into());
                let Some(&chunk_idx) = chunk_index.get(&coord) else {
                    continue;
                };

                let chunk = &editor.buffer.chunks[chunk_idx];

                let render_chunk = if let Some(crc) = self.render_chunks.get_mut(&coord) {
                    crc
                } else {
                    let vao = QuadVao::new(&self.gl, &self.program, &coord, 1)?;
                    let cell_texture = Texture::new_chunk_texture(&self.gl)?;
                    self.render_chunks.insert(
                        coord,
                        RenderChunk {
                            cell_texture,
                            vao,
                            upload: ChunkUploadState::default(),
                        },
                    );
                    self.render_chunks.get_mut(&coord).unwrap()
                };

                let cells = chunk.get_cells();
                let mask_chunk = editor.mask.get_chunk(coord);
                if prepare_chunk_pixels(&mut render_chunk.upload, cells.as_ref(), mask_chunk) {
                    render_chunk
                        .cell_texture
                        .set_pixels(render_chunk.upload.upload_slice())?;
                }

                render_chunk.cell_texture.bind();
                self.program
                    .chunk_start_cell_offset
                    .set(&self.gl, coord.first_cell_coord().0);
                render_chunk.vao.bind();
                self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
            }
        }

        Ok(())
    }
}
