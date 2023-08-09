use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::coords::ChunkCoord;

use super::CellProgram;

pub struct QuadVao {
    ctx: WebGl2RenderingContext,
    vao: WebGlVertexArrayObject,
    buffers: (WebGlBuffer, WebGlBuffer),
}

impl QuadVao {
    pub fn new(
        ctx: &WebGl2RenderingContext,
        program: &CellProgram,
        chunk_coord: &ChunkCoord,
    ) -> Result<QuadVao, String> {
        let l = chunk_coord.0.as_vec2();
        let vertices: [f32; 12] = [
            l.x,
            l.y,
            l.x + 1.0,
            l.y,
            l.x + 1.0,
            l.y + 1.0,
            l.x,
            l.y,
            l.x + 1.0,
            l.y + 1.0,
            l.x,
            l.y + 1.0,
        ];

        let uvs: [f32; 12] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0];

        let vao = ctx
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        ctx.bind_vertex_array(Some(&vao));

        // Positions buffer.
        let position_buffer = ctx.create_buffer().ok_or("Failed to create buffer")?;
        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        unsafe {
            let position_array_buf_view = js_sys::Float32Array::view(&vertices);
            ctx.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &position_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        ctx.enable_vertex_attrib_array(program.attr_position);
        ctx.vertex_attrib_pointer_with_i32(
            program.attr_position,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        // UV buffer.
        let uv_buffer = ctx.create_buffer().ok_or("Failed to create buffer")?;
        ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&uv_buffer));

        unsafe {
            let uv_array_buf_view = js_sys::Float32Array::view(&uvs);
            ctx.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &uv_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        ctx.enable_vertex_attrib_array(program.attr_uv);
        ctx.vertex_attrib_pointer_with_i32(
            program.attr_uv,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        Ok(Self {
            ctx: ctx.clone(),
            vao,
            buffers: (position_buffer, uv_buffer),
        })
    }

    pub fn bind(&self) {
        self.ctx.bind_vertex_array(Some(&self.vao));
    }
}

impl Drop for QuadVao {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.buffers.0));
        self.ctx.delete_buffer(Some(&self.buffers.1));
        self.ctx.delete_vertex_array(Some(&self.vao));
    }
}
