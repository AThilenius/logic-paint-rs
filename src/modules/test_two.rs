use super::{Anchor, Module};

use yew::prelude::*;

#[derive(Clone, Copy)]
pub struct TestTwo {
    pub anchor: Anchor,
}

impl Module for TestTwo {
    fn reset(&mut self) {}

    fn get_anchor(&self) -> Anchor {
        self.anchor
    }

    fn view(&self) -> yew::Html {
        html! {
            <div>{"Module Test Two"}</div>
        }
    }

    fn clone_dyn(&self) -> Box<dyn Module> {
        Box::new(self.clone())
    }
}
