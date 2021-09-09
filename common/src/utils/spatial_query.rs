use bevy::{math::Vec3Swizzles, prelude::*, render::camera::Camera};

use crate::canvas::CanvasData;

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
    world_point: &Vec3,
    canvas: &CanvasData,
    canvas_transform: &GlobalTransform,
) -> Option<IVec2> {
    // Convert the point into the local space of the canvas
    let local_space = canvas_transform.compute_matrix().inverse();
    let local_point = local_space.project_point3(*world_point).xy();

    // In local space the quad extends -0.5 to 0.5 so we can just bump the vector up by 0.5 then
    // multiply by the cell size.
    let scaled = local_point + Vec2::new(0.5, 0.5);

    // Y is inverted in cell coords
    let scaled = Vec2::new(scaled.x, 1.0 - scaled.y);

    let cell_space_point = scaled * Vec2::new(canvas.size as f32, canvas.size as f32);
    let floored = cell_space_point.floor();
    let (x, y) = (floored.x as u32, floored.y as u32);

    if x < canvas.size && y < canvas.size {
        Some(IVec2::new(x as i32, y as i32))
    } else {
        None
    }
}
