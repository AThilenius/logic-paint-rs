mod module_mount;
mod test_one;
mod test_two;

pub use module_mount::*;
pub use test_one::*;
pub use test_two::*;

use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::coords::CellCoord;

pub trait Module {
    fn reset(&mut self);
    fn get_anchor(&self) -> Anchor;
    fn view(&self) -> Html;
    fn clone_dyn(&self) -> Box<dyn Module>;
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub root: CellCoord,
    pub align: Alignment,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    UpperLeft,
    UpperRight,
}
