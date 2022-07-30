use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    utils::Selection, viewport::blueprint::Blueprint, viewport::buffer::Buffer, wgl2::Camera,
};

#[derive(Default)]
pub struct EditorState {
    pub selection: Selection,
    pub camera: Camera,
    pub registers: HashMap<String, Buffer>,
}

#[derive(Serialize, Deserialize)]
pub struct SerdeEditorState {
    pub selection: Selection,
    pub camera_translation_scale: (Vec2, f32),
    pub registers: HashMap<String, Blueprint>,
}

impl From<&EditorState> for SerdeEditorState {
    fn from(editor_state: &EditorState) -> Self {
        Self {
            selection: editor_state.selection.clone(),
            camera_translation_scale: (editor_state.camera.translation, editor_state.camera.scale),
            registers: editor_state
                .registers
                .iter()
                .map(|(name, buffer)| (name.to_owned(), Blueprint::from(buffer)))
                .collect(),
        }
    }
}

impl From<SerdeEditorState> for EditorState {
    fn from(serde_editor_state: SerdeEditorState) -> Self {
        Self {
            selection: serde_editor_state.selection,
            camera: Camera::new_translation_scale(
                serde_editor_state.camera_translation_scale.0,
                serde_editor_state.camera_translation_scale.1,
            ),
            registers: serde_editor_state
                .registers
                .iter()
                .map(|(name, blueprint)| {
                    (
                        name.to_owned(),
                        blueprint
                            .into_buffer_from_partial(&Buffer::default())
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }
}
