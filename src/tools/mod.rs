use crate::{
    substrate::{buffer::Buffer, io::IoState, mask::Mask},
    utils::Selection,
    wgl2::Camera,
};

pub mod camera_controller;
pub mod draw_metal;
pub mod draw_si;
pub mod visual;

pub trait Tool {
    fn activate(&mut self, buffer: Buffer) -> ToolOutput {
        let _ = buffer;
        Default::default()
    }

    fn deactivate(&mut self, buffer: Buffer) -> ToolOutput {
        let _ = buffer;
        Default::default()
    }

    fn dispatch_event(&mut self, input: &ToolInput) -> ToolOutput;
}

pub struct ToolInput {
    /// If this is the active tool. Mostly here as a convenience, tools could of course track
    /// active themselves.
    pub active: bool,
    /// The input state of this event.
    pub io_state: IoState,
    /// The camera currently being used.
    pub camera: Camera,
    /// The editor's buffer.
    pub buffer: Buffer,
    /// The editor's selection.
    pub selection: Selection,
}

#[derive(Default)]
pub struct ToolOutput {
    /// The buffer to persist to the Editor. If set to None, the previously set Buffer remains
    /// active.
    pub buffer: Option<Buffer>,
    /// The mask to persist to the Editor. If set to None, the previously set mask remains active.
    /// Masks are alwasy reset when tools switch.
    pub mask: Option<Mask>,
    /// The camera to persist to the current Viewport. If set to None, the previously set camera
    /// remains active.
    pub camera: Option<Camera>,
    /// What custom CSS cursor the tool would like to switch to.
    pub cursor_style: Option<String>,
    /// When set to true, this is a good time to checkpoint the buffer.
    pub checkpoint: bool,
    /// Allows the tool to 'save' itself to persistent storage.
    pub persist_tool_state: Option<Vec<u8>>,
    /// When set to true, the given tool will become active. Note that `deactivate will be called
    /// on the previously active tool (allowing it to cleanup) before `activate` is called on this
    /// tool. If two tools take active, only the first will become active.
    pub take_active: bool,
    /// The selection to persist to the current Editor and Viewport.
    pub selection: Option<Selection>,
}
