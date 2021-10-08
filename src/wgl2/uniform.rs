use std::marker::PhantomData;

use glam::{IVec2, IVec3, IVec4, Mat4, Vec2, Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

pub struct Uniform<T> {
    location: WebGlUniformLocation,
    _phantom: PhantomData<T>,
}

impl<T> Uniform<T> {
    pub fn new(
        ctx: &WebGl2RenderingContext,
        program: &WebGlProgram,
        name: &str,
    ) -> Result<Self, String> {
        let location = ctx
            .get_uniform_location(&program, name)
            .ok_or_else(|| format!("Failed to find '{}' uniform.", name))?;

        Ok(Self {
            location,
            _phantom: Default::default(),
        })
    }
}

pub trait SetUniformType<T> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: T);
}

impl SetUniformType<i32> for Uniform<i32> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: i32) {
        ctx.uniform1i(Some(&self.location), v);
    }
}

impl SetUniformType<f32> for Uniform<f32> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: f32) {
        ctx.uniform1f(Some(&self.location), v);
    }
}

impl SetUniformType<Vec2> for Uniform<Vec2> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec2) {
        ctx.uniform2f(Some(&self.location), v.x, v.y);
    }
}

impl SetUniformType<IVec2> for Uniform<IVec2> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec2) {
        ctx.uniform2i(Some(&self.location), v.x, v.y);
    }
}

impl SetUniformType<Vec3> for Uniform<Vec3> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec3) {
        ctx.uniform3f(Some(&self.location), v.x, v.y, v.z);
    }
}

impl SetUniformType<IVec3> for Uniform<IVec3> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec3) {
        ctx.uniform3i(Some(&self.location), v.x, v.y, v.z);
    }
}

impl SetUniformType<Vec4> for Uniform<Vec4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec4) {
        ctx.uniform4f(Some(&self.location), v.x, v.y, v.z, v.w);
    }
}

impl SetUniformType<IVec4> for Uniform<IVec4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec4) {
        ctx.uniform4i(Some(&self.location), v.x, v.y, v.z, v.w);
    }
}

impl SetUniformType<Mat4> for Uniform<Mat4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Mat4) {
        ctx.uniform_matrix4fv_with_f32_array(Some(&self.location), false, v.as_ref());
    }
}
