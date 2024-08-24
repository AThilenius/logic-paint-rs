use web_sys::{KeyboardEvent, MouseEvent, WheelEvent};

pub enum RawInput {
    Mouse(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
}
