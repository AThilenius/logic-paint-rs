use bevy::{math::Vec3Swizzles, prelude::*, render::camera::Camera};

use crate::{canvas::Canvas, render::CELL_WORLD_SIZE};

pub fn screen_to_world_point_at_distance(
    pos_screen: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    distance: f32,
) -> Vec3 {
    let camera_position = camera_transform.compute_matrix();
    let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
    let projection_matrix = camera.projection_matrix;

    // Normalized device coordinate cursor position from (-1, -1, -1) to (1, 1, 1)
    let cursor_ndc = (pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
    let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

    // Use near and far ndc points to generate a ray in world space
    // This method is more robust than using the location of the camera as the start of
    // the ray, because ortho cameras have a focal point at infinity!
    let ndc_to_world: Mat4 = camera_position * projection_matrix.inverse();
    let cursor_pos_near: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_near);
    let cursor_pos_far: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_far);
    let ray_direction = cursor_pos_far - cursor_pos_near;

    cursor_pos_near + (ray_direction * distance)
}

pub fn raycast_canvas(
    canvas: &Canvas,
    world_point: &Vec3,
    canvas_transform: &GlobalTransform,
) -> Option<IVec2> {
    // Convert the point into the local space of the canvas
    let local_space = canvas_transform.compute_matrix().inverse();
    let local_point = local_space.project_point3(*world_point).xy();

    // Scale the world_space point down by the size of the quad, and offset it by 0.5 to re-map the
    // x and y range from [-0.5, 0.5] to [0.0, 1.0].
    let quad_size = canvas.size as f32 * CELL_WORLD_SIZE;
    let scaled = (local_point / quad_size) + Vec2::new(0.5, 0.5);

    // Now re-map [0.0, 1.0] range into [0, canvas.size] range ("cell space").
    let cell_space_point = scaled * Vec2::new(canvas.cells.size as f32, canvas.cells.size as f32);
    let floored = cell_space_point.floor();
    let (x, y) = (floored.x as u32, floored.y as u32);

    if x < canvas.cells.size as u32 && y < canvas.cells.size as u32 {
        Some(IVec2::new(x as i32, y as i32))
    } else {
        None
    }
}
