use serde::{Deserialize, Serialize};

use crate::{blueprint::Blueprint, buffer::Buffer, wgl2::Camera};

pub struct Session {
    pub active_buffer: Buffer,
    pub camera: Camera,
}

impl Session {
    pub fn new() -> Self {
        Self {
            active_buffer: Default::default(),
            camera: Camera::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerdeSession {
    active_buffer_blueprint: Blueprint,
    camera: Camera,
}

impl From<&Session> for SerdeSession {
    fn from(session: &Session) -> Self {
        Self {
            active_buffer_blueprint: (&session.active_buffer).into(),
            camera: session.camera.clone(),
        }
    }
}

impl From<&SerdeSession> for Session {
    fn from(serde_session: &SerdeSession) -> Self {
        Self {
            active_buffer: (&serde_session.active_buffer_blueprint).into(),
            camera: serde_session.camera.clone(),
        }
    }
}
