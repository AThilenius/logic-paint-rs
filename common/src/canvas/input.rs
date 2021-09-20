use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    render::camera::Camera,
};

use crate::utils::{raycast_canvas, screen_to_world_point_at_distance};

#[derive(Debug, Default, Clone)]
pub struct CanvasInput {
    pub left_just_pressed: bool,
    pub left_pressed: bool,
    pub right_just_pressed: bool,
    pub right_pressed: bool,
    pub mouse_position: Option<IVec2>,
    pub compile_just_clicked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    NType,
    PType,
    Metal,
    Via,
}

impl Default for ToolType {
    fn default() -> Self {
        Self::NType
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActiveTools {
    pub tool_type: ToolType,
}

pub fn load_canvas_input(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut keyboard_event: EventReader<KeyboardInput>,
    mut canvas_input_query: Query<(&mut CanvasInput, &GlobalTransform)>,
    mut active_tool: ResMut<ActiveTools>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    // Key events we just handle out-of-band. They are considered global data. I'll probably change
    // that later -shrug-.
    let mut compile_just_clicked = false;

    for key_event in keyboard_event.iter() {
        if key_event.state != ElementState::Pressed {
            continue;
        }

        match key_event.key_code {
            Some(KeyCode::Q) => active_tool.tool_type = ToolType::NType,
            Some(KeyCode::W) => active_tool.tool_type = ToolType::PType,
            Some(KeyCode::E) => active_tool.tool_type = ToolType::Metal,
            Some(KeyCode::R) => active_tool.tool_type = ToolType::Via,
            // DEV
            Some(KeyCode::C) => compile_just_clicked = true,
            _ => {}
        }
    }

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

    for (mut canvas_input, global_transform) in canvas_input_query.iter_mut() {
        // Update mouse buttons.
        canvas_input.left_just_pressed = mouse_button.just_pressed(MouseButton::Left);
        canvas_input.left_pressed = mouse_button.pressed(MouseButton::Left);
        canvas_input.right_just_pressed = mouse_button.just_pressed(MouseButton::Right);
        canvas_input.right_pressed = mouse_button.pressed(MouseButton::Right);
        canvas_input.compile_just_clicked = compile_just_clicked;

        // Convert the mouse points into cell space.
        let world_and_cell_positions: Vec<(Vec3, Option<IVec2>)> = world_positions
            .iter()
            .map(|p| (*p, raycast_canvas(p, global_transform)))
            .collect();

        // Check if we need to update mouse_position
        match world_and_cell_positions.last() {
            Some((_, None)) => canvas_input.mouse_position = None,
            Some((_, Some(pos))) => canvas_input.mouse_position = Some(*pos),
            _ => {}
        };
    }
}
