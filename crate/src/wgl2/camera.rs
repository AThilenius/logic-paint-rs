use std::collections::HashSet;

use glam::{IVec2, Mat4, Quat, Vec2, Vec3, Vec3Swizzles};

use crate::{
    coords::{CellCoord, ChunkCoord, CHUNK_SIZE},
    input::InputState,
};

#[derive(Clone, PartialEq)]
pub struct Camera {
    pub translation: Vec2,
    pub scale: f32,
    pub size: Vec2,
    pub proj_matrix: Mat4,
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

    /// Returns true if input processing should be truncated because of panning.
    pub fn handle_input(&mut self, input: &InputState) -> bool {
        // Track the drag-anchor for panning on initial click of Space.
        if input.keydown.contains("Space") {
            self.drag_world_anchor = Some(
                self.drag_world_anchor
                    .unwrap_or_else(|| self.project_screen_point_to_world(input.screen_point)),
            );
        } else {
            self.drag_world_anchor = None;
        }

        // Handle pan mouse dragging. We want to put the drag_world_anchor directly under the mouse.
        let new_world_point = self.project_screen_point_to_world(input.screen_point);
        if let Some(anchor) = self.drag_world_anchor {
            // How far we need to move the camera to move the anchor under the mouse
            self.translation += anchor - new_world_point;

            // Returning here disallows any other camera op while panning. They create
            // discontinuities.
            return true;
        }

        // Handle scroll zooming around the world anchor under the mouse.
        let origin_world = self.project_screen_point_to_world(input.screen_point);
        self.scale += self.scale * input.scroll_delta_y;
        self.scale = f32::clamp(self.scale, 0.02, 10.0);
        self.update_proj_matrix();
        let new_world_point = self.project_screen_point_to_world(input.screen_point);
        self.translation += origin_world - new_world_point;

        false
    }

    /// Project a screen x,y point into the world. Z axis is ignored because I don't need it.
    pub fn project_screen_point_to_world(&self, position: Vec2) -> Vec2 {
        let view_matrix = self.get_view_matrix();

        // Normalized device coordinate cursor position from (-1, -1, -1) to (1, 1, 1). The Y axis
        // is flipped (in HTML Y=0 is the top).
        let mut cursor_ndc = (position / self.size) * 2.0 - Vec2::ONE;
        cursor_ndc.y = -cursor_ndc.y;
        let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
        let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

        // Use near and far ndc points to generate a ray in world space
        let ndc_to_world: Mat4 = view_matrix * self.proj_matrix.inverse();
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

    pub fn project_cell_coord_to_screen_point(&self, coord: CellCoord) -> Vec2 {
        let vec = coord.0.as_vec2() / CHUNK_SIZE as f32;
        let p = self
            .get_view_proj_matrix()
            .project_point3(Vec3::new(vec.x, vec.y, 0.0));
        let half_size = self.size / 2.0;
        (Vec2::new(p.x, -p.y) * half_size) + half_size
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
        v.reserve(
            ((upper_right.0.y - lower_left.0.y) * (upper_right.0.x - lower_left.0.x)) as usize,
        );
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

        // TODO: Something is wrong with this scaling logic. Disabling it fixed the alignment issues
        // with module DOM Nodes though.
        let scale = 720.0 * 2.0; // * self.pixel_ratio;
        let w = self.size.x / scale;
        let h = self.size.y / scale;
        self.proj_matrix = Mat4::orthographic_rh(-w, w, -h, h, 0.0, 1.0);
    }
}
