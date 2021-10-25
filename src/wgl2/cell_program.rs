use glam::Mat4;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use super::uniform::Uniform;

const CELL_VERT_SRC: &str = include_str!("../shaders/cell.vert");
const CELL_FRAG_SRC: &str = include_str!("../shaders/cell.frag");

pub struct CellProgram {
    pub program: WebGlProgram,
    pub view_proj: Uniform<Mat4>,
    pub time: Uniform<f32>,
    pub cells_texture_sampler: Uniform<i32>,
    pub attr_position: u32,
    pub attr_uv: u32,
}

impl CellProgram {
    pub fn compile(ctx: &WebGl2RenderingContext) -> Result<CellProgram, String> {
        let vert_shader =
            compile_shader(&ctx, WebGl2RenderingContext::VERTEX_SHADER, CELL_VERT_SRC)?;
        let frag_shader =
            compile_shader(&ctx, WebGl2RenderingContext::FRAGMENT_SHADER, CELL_FRAG_SRC)?;
        let program: WebGlProgram = link_program(&ctx, &vert_shader, &frag_shader)?;
        ctx.use_program(Some(&program));
        let attr_position = ctx.get_attrib_location(&program, "position") as u32;
        let attr_uv = ctx.get_attrib_location(&program, "uv") as u32;

        Ok(Self {
            program: program.clone(),
            view_proj: Uniform::new(&ctx, &program, "view_proj"),
            time: Uniform::new(&ctx, &program, "time"),
            cells_texture_sampler: Uniform::new(&ctx, &program, "cells_texture_sampler"),
            attr_position,
            attr_uv,
        })
    }

    pub fn use_program(&self, ctx: &WebGl2RenderingContext) {
        ctx.use_program(Some(&self.program));
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
