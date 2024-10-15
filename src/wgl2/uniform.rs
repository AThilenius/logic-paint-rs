use std::marker::PhantomData;

use glam::{IVec2, IVec3, IVec4, Mat4, Vec2, Vec3, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::warn;

pub struct Uniform<T> {
    location: Option<WebGlUniformLocation>,
    _phantom: PhantomData<T>,
}

impl<T> Uniform<T> {
    pub fn new(ctx: &WebGl2RenderingContext, program: &WebGlProgram, name: &str) -> Self {
        let location = ctx.get_uniform_location(&program, name);

        if location.is_none() {
            warn!(r#"Failed to find uniform "{}". It will be ignored."#, name);
        }

        Self {
            location,
            _phantom: Default::default(),
        }
    }
}

pub trait SetUniformType<T> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: T);
}

impl SetUniformType<u32> for Uniform<u32> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: u32) {
        if let Some(location) = &self.location {
            ctx.uniform1ui(Some(&location), v);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<i32> for Uniform<i32> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: i32) {
        if let Some(location) = &self.location {
            ctx.uniform1i(Some(&location), v);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<f32> for Uniform<f32> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: f32) {
        if let Some(location) = &self.location {
            ctx.uniform1f(Some(&location), v);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<Vec2> for Uniform<Vec2> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec2) {
        if let Some(location) = &self.location {
            ctx.uniform2f(Some(&location), v.x, v.y);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<IVec2> for Uniform<IVec2> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec2) {
        if let Some(location) = &self.location {
            ctx.uniform2i(Some(&location), v.x, v.y);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<Vec3> for Uniform<Vec3> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec3) {
        if let Some(location) = &self.location {
            ctx.uniform3f(Some(&location), v.x, v.y, v.z);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<IVec3> for Uniform<IVec3> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec3) {
        if let Some(location) = &self.location {
            ctx.uniform3i(Some(&location), v.x, v.y, v.z);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<Vec4> for Uniform<Vec4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Vec4) {
        if let Some(location) = &self.location {
            ctx.uniform4f(Some(&location), v.x, v.y, v.z, v.w);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<IVec4> for Uniform<IVec4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: IVec4) {
        if let Some(location) = &self.location {
            ctx.uniform4i(Some(&location), v.x, v.y, v.z, v.w);
        } else {
            warn!("Program uniform location missing");
        }
    }
}

impl SetUniformType<Mat4> for Uniform<Mat4> {
    fn set(&self, ctx: &WebGl2RenderingContext, v: Mat4) {
        if let Some(location) = &self.location {
            ctx.uniform_matrix4fv_with_f32_array(Some(&location), false, v.as_ref());
        } else {
            warn!("Program uniform location missing");
        }
    }
}
