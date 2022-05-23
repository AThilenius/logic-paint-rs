use glam::{Mat4, Vec2, Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use super::{uniform::Uniform, SetUniformType};

const CELL_VERT_SRC: &str = include_str!("../shaders/cell.vert");
const CELL_FRAG_SRC: &str = include_str!("../shaders/cell.frag");

pub struct CellProgram {
    pub program: WebGlProgram,
    pub view_proj: Uniform<Mat4>,
    pub time: Uniform<f32>,
    pub cells_texture_sampler: Uniform<i32>,
    pub mask_texture_sampler: Uniform<i32>,
    pub attr_position: u32,
    pub attr_uv: u32,

    // Style uniforms
    pub n_color: Uniform<Vec4>,
    pub p_color: Uniform<Vec4>,
    pub metal_color: Uniform<Vec3>,
    pub io_color: Uniform<Vec3>,
    pub active_color: Uniform<Vec3>,
    pub grid_color: Uniform<Vec3>,
    pub background_color: Uniform<Vec3>,
    pub grid_res: Uniform<Vec2>,
    pub grid_blend_strength: Uniform<f32>,
    pub metal_over_si_blend: Uniform<f32>,
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
        let cells_texture_sampler = Uniform::new(&ctx, &program, "cells_texture_sampler");
        let mask_texture_sampler = Uniform::new(&ctx, &program, "mask_texture_sampler");

        let n_color = Uniform::new(&ctx, &program, "n_color");
        let p_color = Uniform::new(&ctx, &program, "p_color");
        let metal_color = Uniform::new(&ctx, &program, "metal_color");
        let io_color = Uniform::new(&ctx, &program, "io_color");
        let active_color = Uniform::new(&ctx, &program, "active_color");
        let grid_color = Uniform::new(&ctx, &program, "grid_color");
        let background_color = Uniform::new(&ctx, &program, "background_color");
        let grid_res = Uniform::new(&ctx, &program, "grid_res");
        let grid_blend_strength = Uniform::new(&ctx, &program, "grid_blend_strength");
        let metal_over_si_blend = Uniform::new(&ctx, &program, "metal_over_si_blend");

        // Set default style values
        n_color.set(&ctx, Vec4::new(0.98, 0.0, 0.77, 1.0));
        p_color.set(&ctx, Vec4::new(0.0, 0.87, 1.0, 1.0));
        metal_color.set(&ctx, Vec3::new(0.2, 0.2, 0.2));
        io_color.set(&ctx, Vec3::new(0.3, 0.3, 0.3));
        active_color.set(&ctx, Vec3::new(1.0, 1.0, 1.0));
        grid_color.set(&ctx, Vec3::new(1.0, 1.0, 1.0));
        background_color.set(&ctx, Vec3::new(0.0, 0.0, 0.0));
        grid_res.set(&ctx, Vec2::new(32.0, 32.0));
        grid_blend_strength.set(&ctx, 0.03);
        metal_over_si_blend.set(&ctx, 0.6);

        // Set defaults for texture samplers
        cells_texture_sampler.set(&ctx, 0);
        mask_texture_sampler.set(&ctx, 1);

        Ok(Self {
            program: program.clone(),
            view_proj: Uniform::new(&ctx, &program, "view_proj"),
            time: Uniform::new(&ctx, &program, "time"),
            cells_texture_sampler,
            mask_texture_sampler,
            attr_position,
            attr_uv,
            n_color,
            p_color,
            metal_color,
            io_color,
            active_color,
            grid_color,
            background_color,
            grid_res,
            grid_blend_strength,
            metal_over_si_blend,
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