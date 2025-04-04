use std::collections::HashSet;

use glam::{IVec2, Mat4, Quat, Vec2, Vec3, Vec3Swizzles};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::coords::{CellCoord, ChunkCoord, CHUNK_SIZE};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[wasm_bindgen]
pub struct Camera {
    pub translation: Vec2,
    pub scale: f32,
    pub size: Vec2,
    #[wasm_bindgen(skip)]
    pub proj_matrix: Mat4,
    drag_world_anchor: Option<Vec2>,
}

impl PartialEq for Camera {
    fn eq(&self, other: &Self) -> bool {
        self.translation
            .abs_diff_eq(other.translation, f32::EPSILON)
            && self.size.abs_diff_eq(other.size, f32::EPSILON)
            && f32::abs(self.scale - other.scale) < f32::EPSILON
    }
}

impl Default for Camera {
    fn default() -> Self {
        let mut camera = Self {
            translation: Vec2::ZERO,
            scale: 1.0,
            size: Vec2::ONE,
            proj_matrix: Default::default(),
            drag_world_anchor: None,
        };

        camera.update_proj_matrix();
        camera
    }
}

// Non-bound methods
impl Camera {
    pub fn update(&mut self, size: Vec2) {
        if !self.size.abs_diff_eq(size, f32::EPSILON) {
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

    fn update_proj_matrix(&mut self) {
        let scale = 2000.0;
        let w = self.size.x / scale;
        let h = self.size.y / scale;
        self.proj_matrix = Mat4::orthographic_rh(-w, w, -h, h, 0.0, 1.0);
    }
}

#[wasm_bindgen]
impl Camera {
    #[wasm_bindgen(constructor)]
    pub fn new_translation_scale(translation: Vec2, scale: f32) -> Self {
        let mut camera = Self {
            translation,
            scale,
            size: Vec2::ONE,
            proj_matrix: Default::default(),
            drag_world_anchor: None,
        };

        camera.update_proj_matrix();
        camera
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
}
