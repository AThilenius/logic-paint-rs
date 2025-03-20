use glam::Vec2;

use super::{Tool, ToolInput, ToolOutput};

#[derive(Default)]
pub struct ToolCameraController {
    drag_world_anchor: Option<Vec2>,
}

impl Tool for ToolCameraController {
    fn get_name(&self) -> &str {
        "camera-controller"
    }

    fn dispatch_event(
        &mut self,
        ToolInput {
            io_state, camera, ..
        }: &ToolInput,
    ) -> ToolOutput {
        let mut camera = camera.clone();

        // Track the drag-anchor for panning on initial click of Space.
        if io_state.get_key_code("Space").down || io_state.scroll_button.down {
            self.drag_world_anchor =
                Some(self.drag_world_anchor.unwrap_or_else(|| {
                    camera.project_screen_point_to_world(io_state.screen_point)
                }));
        } else {
            self.drag_world_anchor = None;
        }

        // Handle pan mouse dragging. We want to put the drag_world_anchor directly under the mouse.
        let new_world_point = camera.project_screen_point_to_world(io_state.screen_point);
        if let Some(anchor) = self.drag_world_anchor {
            // How far we need to move the camera to move the anchor under the mouse
            camera.translation += anchor - new_world_point;
        } else {
            // Handle scroll zooming around the world anchor under the mouse.
            let origin_world = camera.project_screen_point_to_world(io_state.screen_point);
            camera.scale += camera.scale * io_state.scroll_delta_y;
            camera.scale = f32::clamp(camera.scale, 0.04, 40.0);
            let new_world_point = camera.project_screen_point_to_world(io_state.screen_point);
            camera.translation += origin_world - new_world_point;
        }

        ToolOutput {
            camera: Some(camera),
            ..Default::default()
        }
    }
}
