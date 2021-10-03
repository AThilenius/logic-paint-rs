use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlVertexArrayObject};

pub struct QuadVao {
    vao: WebGlVertexArrayObject,
}

impl QuadVao {
    pub fn new(ctx: &WebGl2RenderingContext, program: &WebGlProgram) -> Result<QuadVao, String> {
        // Used for both position and UV coords.
        let vertices: [f32; 12] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0];

        let vao = ctx
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        ctx.bind_vertex_array(Some(&vao));

        let position_uv_buffer = ctx.create_buffer().ok_or("Failed to create buffer")?;
        ctx.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&position_uv_buffer),
        );

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let position_uv_array_buf_view = js_sys::Float32Array::view(&vertices);

            ctx.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &position_uv_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let position_uv_attribute_location =
            ctx.get_attrib_location(&program, "position_uv") as u32;
        ctx.enable_vertex_attrib_array(position_uv_attribute_location);
        ctx.vertex_attrib_pointer_with_i32(
            position_uv_attribute_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        Ok(Self { vao })
    }
}
