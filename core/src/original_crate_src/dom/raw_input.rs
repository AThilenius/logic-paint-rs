use yew::prelude::*;

pub enum RawInput {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
}

// Input Events:

// Mouse
//   - primary/secondary down: bool
//   - left/right: f32
//   - wheel delta: Vec2
// Keyboard
//   - code: String
//   - key: String
//   - repeat: bool
// Resize
//   - width/height: u32
// Focus
//   - focused: bool
//     - Used to reset the selected tool and state (except running)

// Output Events:

// Editor State Change
// Used to persist the editor state during an active session. This is really nice when flipping
// back/forth between blueprints.
//   - Selection
//   - Camera Translation/Scale
//   - Registers
