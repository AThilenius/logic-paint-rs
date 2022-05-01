use std::{cell::RefCell, rc::Rc};

use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    blueprint::Blueprint,
    brush::Brush,
    buffer::Buffer,
    buffer_mask::BufferMask,
    coords::CellCoord,
    modules::{Alignment, Anchor, ModuleData, TogglePinData},
    wgl2::Camera,
};

/// The software analogy would be a git repo + working directory.
///
/// The outermost serializable Logic Paint object, represents a history of human work on a design.
/// However, this history might be ephemeral if a session is being used to load a Blueprint.
///
/// Additionally, a Session object stores (but does not serialize) data associated with the current
/// editing session (equivalent of a git working directory). This includes data like the copy-paste
/// buffers and active masks.
pub struct Session {
    pub active_buffer: Buffer,
    pub active_mask: BufferMask,
    pub camera: Camera,
    pub brush: Brush,
}

impl Session {
    pub fn new() -> Self {
        let fake_modules = vec![
            ModuleData::TogglePin(Rc::new(RefCell::new(TogglePinData {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(0, 4)),
                    align: Alignment::BottomLeft,
                },
                active: false,
            }))),
            // Rc::new(RefCell::new(Module::TestOne(TestOne {
            //     anchor: Anchor {
            //         root: CellCoord(IVec2::new(20, 4)),
            //         align: Alignment::TopLeft,
            //     },
            //     time: 0.0,
            // }))),
            // Rc::new(RefCell::new(Module::TestTwo(TestTwo {
            //     anchor: Anchor {
            //         root: CellCoord(IVec2::new(0, -1)),
            //         align: Alignment::TopLeft,
            //     },
            //     text: "Top Left".to_string(),
            // }))),
            // Rc::new(RefCell::new(Module::TestTwo(TestTwo {
            //     anchor: Anchor {
            //         root: CellCoord(IVec2::new(-1, -1)),
            //         align: Alignment::TopRight,
            //     },
            //     text: "Top Right".to_string(),
            // }))),
            // Rc::new(RefCell::new(Module::TestTwo(TestTwo {
            //     anchor: Anchor {
            //         root: CellCoord(IVec2::new(0, 0)),
            //         align: Alignment::BottomLeft,
            //     },
            //     text: "Bottom Left".to_string(),
            // }))),
            // Rc::new(RefCell::new(Module::TestTwo(TestTwo {
            //     anchor: Anchor {
            //         root: CellCoord(IVec2::new(-1, 0)),
            //         align: Alignment::BottomRight,
            //     },
            //     text: "Bottom Right".to_string(),
            // }))),
        ];

        let mut active_buffer = Buffer::default();
        active_buffer.transaction_begin();
        for module in fake_modules.into_iter() {
            active_buffer.transact_set_module(module);
        }
        active_buffer.transaction_commit(false);

        Self {
            active_buffer,
            active_mask: BufferMask::default(),
            camera: Camera::new(),
            brush: Brush::new(),
        }
    }

    /// Called once per frame regardless of execution state.
    /// TODO: This probably shouldn't exist?
    pub fn update(&mut self, time: f64) {
        for module in self.active_buffer.get_modules().iter_mut() {
            module.update(time);
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
            active_mask: BufferMask::default(),
            camera: serde_session.camera.clone(),
            brush: Brush::new(),
        }
    }
}
