mod module_mount;
mod test_one;
mod test_two;

pub use module_mount::*;
pub use test_one::*;
pub use test_two::*;

use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::coords::CellCoord;

/// Represents a single module mounted to an `Anchor` within a `Buffer`.
///
/// An enum is used instead of a trait object for several reasons: the interaction with Yew code,
/// fast module access with the critical path in the sim loo, to support frequent copying, and
/// serialization.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Module {
    TestOne(TestOne),
    TestTwo(TestTwo),
}

impl Module {
    fn reset(&mut self) {
        match self {
            Module::TestOne(m) => m.reset(),
            Module::TestTwo(m) => m.reset(),
        }
    }

    fn get_anchor(&self) -> Anchor {
        match self {
            Module::TestOne(m) => m.get_anchor(),
            Module::TestTwo(m) => m.get_anchor(),
        }
    }

    fn view(&self) -> Html {
        match self {
            Module::TestOne(m) => m.view(),
            Module::TestTwo(m) => m.view(),
        }
    }
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
