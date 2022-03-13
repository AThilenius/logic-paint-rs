use super::Anchor;

use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TestTwo {
    pub anchor: Anchor,
}

impl TestTwo {
    pub fn reset(&mut self) {}

    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn view(&self) -> yew::Html {
        html! {
            <div>{"Module Test Two"}</div>
        }
    }
}
