use std::collections::HashSet;

use glam::{IVec2, Mat4, Quat, Vec2, Vec3, Vec3Swizzles};
use serde::{Deserialize, Serialize};

use crate::{
    coords::{CellCoord, ChunkCoord, CHUNK_SIZE},
    dom::ElementInputEvent,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Camera {
    pub translation: Vec2,
    pub scale: f32,
    pub proj_matrix: Mat4,
    pub size: Vec2,
    pixel_ratio: f32,
    drag_world_anchor: Option<Vec2>,
}

impl Camera {
    pub fn new() -> Self {
        let mut camera = Self {
            translation: Vec2::ZERO,
            scale: 1.0,
            pixel_ratio: 1.0,
            size: Vec2::ONE,
            proj_matrix: Default::default(),
            drag_world_anchor: None,
        };

        camera.update_proj_matrix();
        camera
    }

    pub fn update(&mut self, pixel_ratio: f32, size: Vec2) {
        if self.pixel_ratio != pixel_ratio || self.size != size {
            self.pixel_ratio = pixel_ratio;
            self.size = size;
            self.update_proj_matrix();
        }
    }

    pub fn get_view_proj_matrix(&self) -> Mat4 {
        self.proj_matrix * self.get_view_matrix().inverse()
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.scale,
            Quat::IDENTITY,
            Vec3::new(self.translation.x, self.translation.y, 0.0),
        )
    }

    pub fn handle_mouse_event(&mut self, event: ElementInputEvent) {
        match event {
            ElementInputEvent::MouseWheelEvent(event) => {
                // Zoom centered around the cursor
                let screen_point = Vec2::new(event.offset_x() as f32, event.offset_y() as f32);
                let origin_world = self.project_screen_point_to_world(screen_point);
                self.scale += self.scale * event.delta_y() as f32 * 0.001;
                self.scale = f32::clamp(self.scale, 0.02, 10.0);
                self.update_proj_matrix();
                let new_world_point = self.project_screen_point_to_world(screen_point);
                self.translation += origin_world - new_world_point;
            }
            ElementInputEvent::MouseDown(event) if event.button() == 1 => {
                self.drag_world_anchor = Some(self.project_screen_point_to_world(Vec2::new(
                    event.offset_x() as f32,
                    event.offset_y() as f32,
                )));
            }
            ElementInputEvent::MouseUp(event) if event.button() == 1 => {
                self.drag_world_anchor = None;
            }
            ElementInputEvent::MouseMove(event) if event.buttons() & 4 != 0 => {
                // We want to put the drag_world_anchor directly under the mouse.
                let new_world_point = self.project_screen_point_to_world(Vec2::new(
                    event.offset_x() as f32,
                    event.offset_y() as f32,
                ));
                if let Some(anchor) = self.drag_world_anchor {
                    // How far do we need to move the camera to move the anchor under the mouse
                    self.translation += anchor - new_world_point;
                }
            }
            _ => {}
        }
    }

    /// Project a screen x,y point into the world. Z axis is ignored because I don't need it.
    pub fn project_screen_point_to_world(&self, position: Vec2) -> Vec2 {
        let camera_position = self.get_view_matrix();

        // Normalized device coordinate cursor position from (-1, -1, -1) to (1, 1, 1). The Y axis
        // is flipped (in HTML Y=0 is the top).
        let mut cursor_ndc = (position / self.size) * 2.0 - Vec2::ONE;
        cursor_ndc.y = -cursor_ndc.y;
        let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
        let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

        // Use near and far ndc points to generate a ray in world space
        // This method is more robust than using the location of the camera as the start of
        // the ray, because ortho cameras have a focal point at infinity!
        let ndc_to_world: Mat4 = camera_position * self.proj_matrix.inverse();
        let cursor_pos_near: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_near);
        let cursor_pos_far: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_far);
        let ray_direction = cursor_pos_far - cursor_pos_near;

        // Leaving this in just incase I care about distance some day -shrug-
        let distance = 1.0;
        let point = cursor_pos_near + (ray_direction * distance);

        point.xy()
    }

    /// Project a screen point to a cell location. It's the caller's responsibility to ensure the
    /// point is within the visible bounds of the window.
    pub fn project_screen_point_to_cell(&self, position: Vec2) -> CellCoord {
        let world_point = self.project_screen_point_to_world(position);

        // A single chunk is always 1.0 x 1.0 in world coords, and has CHUNK_SIZE x CHUNK_SIZE cells
        // in it. Aka, there are CHUNK_SIZE cells per world-space unit. This makes the math pretty
        // easy from world-space then.
        CellCoord((world_point * CHUNK_SIZE as f32).floor().as_ivec2())
    }

    /// Returns a list of all currently-visible substrate chunks to this camera.
    pub fn get_visible_chunk_coords(&self) -> HashSet<ChunkCoord> {
        let lower_left: ChunkCoord = self
            .project_screen_point_to_cell(Vec2::new(-1.0, self.size.y + 1.0))
            .into();
        let upper_right: ChunkCoord = self
            .project_screen_point_to_cell(Vec2::new(self.size.x + 1.0, -1.0))
            .into();

        let mut v = HashSet::new();
        for y in lower_left.0.y..(upper_right.0.y + 1) {
            for x in lower_left.0.x..(upper_right.0.x + 1) {
                v.insert(ChunkCoord(IVec2::new(x, y)));
            }
        }

        v
    }

    fn update_proj_matrix(&mut self) {
        // Use a DPI-like scaling, so that the scale doesn't change with screen size, only the
        // amount visible.
        let scale = 720.0 * 2.0 * self.pixel_ratio;
        let w = self.size.x / scale;
        let h = self.size.y / scale;
        self.proj_matrix = Mat4::orthographic_rh(-w, w, -h, h, 0.0, 1.0);
    }
}
