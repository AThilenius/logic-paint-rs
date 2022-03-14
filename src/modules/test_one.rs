use super::Anchor;

use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TestOne {
    pub anchor: Anchor,

    #[serde(skip)]
    pub time: f64,
}

impl TestOne {
    pub fn reset(&mut self) {}

    pub fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn view(&self) -> yew::Html {
        html! {
            <div>{format!("Time: {:.2}", self.time)}</div>
        }
    }

    pub fn update(&mut self, time: f64) {
        self.time = time;
    }
}
