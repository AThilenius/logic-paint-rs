use glam::Mat4;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

const CELL_VERT_SRC: &str = include_str!("../shaders/cell.vert");
const CELL_FRAG_SRC: &str = include_str!("../shaders/cell.frag");

pub struct CellProgram {
    pub program: WebGlProgram,
    pub uniform_loc_view_proj: WebGlUniformLocation,
    pub uniform_loc_model: WebGlUniformLocation,
}

impl CellProgram {
    pub fn compile(ctx: &WebGl2RenderingContext) -> Result<CellProgram, String> {
        let vert_shader =
            compile_shader(&ctx, WebGl2RenderingContext::VERTEX_SHADER, CELL_VERT_SRC)?;
        let frag_shader =
            compile_shader(&ctx, WebGl2RenderingContext::FRAGMENT_SHADER, CELL_FRAG_SRC)?;
        let program = link_program(&ctx, &vert_shader, &frag_shader)?;
        let uniform_loc_view_proj = ctx
            .get_uniform_location(&program, "view_proj")
            .ok_or_else(|| String::from("Failed to find 'view_proj' uniform"))?;
        let uniform_loc_model = ctx
            .get_uniform_location(&program, "model")
            .ok_or_else(|| String::from("Failed to find 'model' uniform"))?;
        Ok(Self {
            program,
            uniform_loc_view_proj,
            uniform_loc_model,
        })
    }

    pub fn use_program(&self, ctx: &WebGl2RenderingContext) {
        ctx.use_program(Some(&self.program));
    }

    pub fn set_view_proj(
        &self,
        ctx: &WebGl2RenderingContext,
        width: u32,
        height: u32,
        view_mat: Mat4,
    ) {
        let proj_mat = if width > height {
            let aspect = width as f32 / height as f32;
            Mat4::orthographic_rh(-aspect, aspect, -1.0, 1.0, 0.0, 1.0)
        } else {
            let aspect = height as f32 / width as f32;
            Mat4::orthographic_rh(-1.0, 1.0, -aspect, aspect, 0.0, 1.0)
        };
        let view_proj_mat = proj_mat * view_mat;
        ctx.uniform_matrix4fv_with_f32_array(
            Some(&self.uniform_loc_view_proj),
            false,
            view_proj_mat.as_ref(),
        );
    }

    pub fn set_model(&self, ctx: &WebGl2RenderingContext, mat: Mat4) {
        ctx.uniform_matrix4fv_with_f32_array(Some(&self.uniform_loc_model), false, mat.as_ref());
    }
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
