use yew::prelude::*;

pub enum RawInput {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    MouseWheelEvent(WheelEvent),
    KeyPressed(KeyboardEvent),
}
