use std::{cell::RefCell, rc::Rc};

use glam::IVec2;
use serde::{Deserialize, Serialize};

use crate::{
    blueprint::Blueprint,
    brush::Brush,
    buffer::Buffer,
    coords::CellCoord,
    modules::{Alignment, Anchor, Module, TestOne, TestTwo},
    wgl2::Camera,
};

/// Represents a single editing/run session, which can be stored and reloaded later. This the
/// outermost object that is serialized and stored in Logic Paint. Session can be associated with an
/// editing thread (a human has edited the same session over the course of N days) or they can be
/// ephemeral when used to simply run a Blueprint.
pub struct Session {
    pub active_buffer: Buffer,
    pub camera: Camera,
    pub brush: Brush,
    pub modules: Vec<Rc<RefCell<Module>>>,
}

impl Session {
    pub fn new() -> Self {
        let fake_modules = vec![
            Rc::new(RefCell::new(Module::TestOne(TestOne {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(20, 4)),
                    align: Alignment::TopLeft,
                },
                time: 0.0,
            }))),
            Rc::new(RefCell::new(Module::TestTwo(TestTwo {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(0, -1)),
                    align: Alignment::TopLeft,
                },
                text: "Top Left".to_string(),
            }))),
            Rc::new(RefCell::new(Module::TestTwo(TestTwo {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(-1, -1)),
                    align: Alignment::TopRight,
                },
                text: "Top Right".to_string(),
            }))),
            Rc::new(RefCell::new(Module::TestTwo(TestTwo {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(0, 0)),
                    align: Alignment::BottomLeft,
                },
                text: "Bottom Left".to_string(),
            }))),
            Rc::new(RefCell::new(Module::TestTwo(TestTwo {
                anchor: Anchor {
                    root: CellCoord(IVec2::new(-1, 0)),
                    align: Alignment::BottomRight,
                },
                text: "Bottom Right".to_string(),
            }))),
        ];

        Self {
            active_buffer: Default::default(),
            camera: Camera::new(),
            brush: Brush::new(),
            modules: fake_modules,
        }
    }

    /// Called once per frame regardless of execution state.
    /// TODO: This probably shouldn't exist?
    pub fn update(&mut self, time: f64) {
        for module in self.modules.iter() {
            module.borrow_mut().update(time);
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
