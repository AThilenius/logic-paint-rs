use bevy::{prelude::*, render::camera::Camera};
use fast_voxel_traversal::raycast_2d::{BoundingVolume2, Ray2};

use crate::{
    canvas::Canvas,
    utils::spatial_query::{raycast_canvas, screen_to_world_point_at_distance},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CanvasInput {
    pub left_just_pressed: bool,
    pub left_pressed: bool,
    pub right_just_pressed: bool,
    pub right_pressed: bool,
    pub mouse_position: Option<IVec2>,
    pub mouse_moved: Vec<IVec2>,
}

pub fn load_canvas_input(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut canvas_query: Query<(&mut CanvasInput, &Canvas, &GlobalTransform)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    let world_positions = match camera_query.single() {
        Ok((camera, camera_transform)) => {
            let window = windows
                .get(camera.window)
                .expect("Failed to get camera's window");

            cursor_moved_events
                .iter()
                .map(|event| {
                    screen_to_world_point_at_distance(
                        event.position,
                        window,
                        camera,
                        camera_transform,
                        1.0,
                    )
                })
                .collect()
        }
        Err(_) => vec![],
    };

    for (mut canvas_input, canvas, global_transform) in canvas_query.iter_mut() {
        // Update mouse buttons.
        canvas_input.left_just_pressed = mouse_button.just_pressed(MouseButton::Left);
        canvas_input.left_pressed = mouse_button.pressed(MouseButton::Left);
        canvas_input.right_just_pressed = mouse_button.just_pressed(MouseButton::Right);
        canvas_input.right_pressed = mouse_button.pressed(MouseButton::Right);

        // Convert the mouse points into cell space.
        let world_and_cell_positions: Vec<(Vec3, Option<IVec2>)> = world_positions
            .iter()
            .map(|p| (*p, raycast_canvas(p, &canvas, global_transform)))
            .collect();

        // Store the previous point (if any) for raycasting
        let previous = canvas_input.mouse_position;

        // Check if we need to update mouse_position
        match world_and_cell_positions.last() {
            Some((_, None)) => canvas_input.mouse_position = None,
            Some((_, Some(pos))) => canvas_input.mouse_position = Some(*pos),
            _ => {}
        };

        // Now we want to update all the "moved" data. But we won't get mouse events at a high
        // enough frequency to guarantee each and every cell gets a "hit" as the mouse moves. So
        // instead we raycast between each mouse event to find all the cells that could have been
        // hit. However, we need to skip events that missed the canvas and not just cull them from
        // the list.
        canvas_input.mouse_moved.clear();
        let volume = BoundingVolume2 {
            size: (canvas.size as i32, canvas.size as i32),
        };

        // Still no if-let chaining in Rust -cry- https://github.com/rust-lang/rust/issues/53667
        if let (Some(previous), Some((_, Some(first)))) =
            (previous, world_and_cell_positions.first())
        {
            if previous != *first {
                let dir = (*first - previous).as_f32();
                let ray = Ray2 {
                    origin: previous.as_f32().into(),
                    direction: dir.into(),
                    length: dir.length(),
                };
                canvas_input
                    .mouse_moved
                    .extend(volume.traverse_ray(ray).map(|hit| IVec2::from(hit.voxel)));
            }
        }

        for i in 0..world_and_cell_positions.len().saturating_sub(1) {
            if let (Some((_, Some(first))), Some((_, Some(second)))) = (
                world_and_cell_positions.get(i),
                world_and_cell_positions.get(i + 1),
            ) {
                if first != second {
                    let dir = (*second - *first).as_f32();
                    let ray = Ray2 {
                        origin: first.as_f32().into(),
                        direction: dir.into(),
                        length: dir.length(),
                    };
                    canvas_input
                        .mouse_moved
                        .extend(volume.traverse_ray(ray).map(|hit| IVec2::from(hit.voxel)));
                }
            }
        }
    }
}
