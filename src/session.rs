use serde::{Deserialize, Serialize};

use crate::wgl2::Camera;

use super::buffer::Buffer;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub active_buffer: Buffer,
    camera: Camera,
}

impl Session {
    pub fn new() -> Self {
        Self {
            active_buffer: Default::default(),
            camera: Camera::new(),
        }
    }
}
