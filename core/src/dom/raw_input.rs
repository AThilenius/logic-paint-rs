use yew::prelude::*;

pub enum RawInput {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
}

// It would be really nice if registers were external. Including the mouse follow. I remember that
// being a pain in the ass. It'll let you move stuff back and forth between editors a lot easier. I
// could just throw them into LocalStorage 🤔 That's awfully easy. It has the advantage of
// persisting through a refresh too.

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
// - Used primarily to reset the editor back to visual mode when focus is lost.
// - Has
//   - focused: bool
// Set Pins
//   - pins: Vec<Pin>
// Set Editor State
// - Set the overall state of the editor itself (not the blueprint). This is useful for (example)
//   UGH. Hold up.

// Output Events:

// Editor State Change
// -  Used to persist the editor state during an active session. This is really nice when flipping
//    back/forth between blueprints.
// - Has
//   - Selection
//   - Camera Translation/Scale
//   - Registers (including mouse follow)

// Pin State
// Used to update the pin state for external modules.
