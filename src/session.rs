use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    blueprint::Blueprint,
    brush::Brush,
    buffer::Buffer,
    coords::CellCoord,
    module::{Module, ModuleAlignment, ModuleMount},
    wgl2::Camera,
};

pub struct Session {
    pub active_buffer: Buffer,
    pub camera: Camera,
    pub brush: Brush,
    pub modules: Vec<ModuleMount>,
}

impl Session {
    pub fn new() -> Self {
        let fake_modules: Vec<ModuleMount> = vec![
            (
                CellCoord(IVec2::new(0, 0)),
                ModuleAlignment::UpperRight,
                Module::One {
                    string: "Hello, string".to_string(),
                },
            )
                .into(),
            (
                CellCoord(IVec2::new(10, 10)),
                ModuleAlignment::UpperLeft,
                Module::Two {
                    number: 42,
                    vec: vec![1, 2, 3, 4],
                },
            )
                .into(),
        ];

        Self {
            active_buffer: Default::default(),
            camera: Camera::new(),
            brush: Brush::new(),
            modules: fake_modules,
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
            brush: Brush::new(),
            // TODO:
            modules: vec![],
        }
    }
}