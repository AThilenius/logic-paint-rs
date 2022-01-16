use super::{Anchor, Module};

use yew::prelude::*;

#[derive(Clone, Copy)]
pub struct TestOne {
    pub anchor: Anchor,
}

impl Module for TestOne {
    fn reset(&mut self) {}

    fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    fn view(&self) -> yew::Html {
        html! {
            <div>{"Module Test One"}</div>
        }
    }

    fn clone_dyn(&self) -> Box<dyn Module> {
        Box::new(self.clone())
    }
}
