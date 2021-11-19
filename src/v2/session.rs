use super::{buffer::Buffer, render_context::RenderContext};

pub struct Session {
    pub active_buffer: Buffer,
}

impl Session {
    pub fn new() -> Self {
        Self {
            active_buffer: Default::default(),
        }
    }
}
