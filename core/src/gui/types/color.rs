use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[allow(unused)]
impl Color {
    pub const AQUAMARINE: Self = Self::new(127, 255, 212, 255);
    pub const BEIGE: Self = Self::new(245, 245, 220, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const BROWN: Self = Self::new(165, 42, 42, 255);
    pub const CORAL: Self = Self::new(255, 127, 80, 255);
    pub const CYAN: Self = Self::new(0, 255, 255, 255);
    pub const DARK_GRAY: Self = Self::new(169, 169, 169, 255);
    pub const FOREST: Self = Self::new(34, 139, 34, 255);
    pub const GOLD: Self = Self::new(255, 215, 0, 255);
    pub const GRAY: Self = Self::new(128, 128, 128, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const INDIGO: Self = Self::new(75, 0, 130, 255);
    pub const KHAKI: Self = Self::new(240, 230, 140, 255);
    pub const LAVENDER: Self = Self::new(230, 230, 250, 255);
    pub const LIGHT_GRAY: Self = Self::new(211, 211, 211, 255);
    pub const LIME: Self = Self::new(0, 255, 0, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    pub const MAROON: Self = Self::new(128, 0, 0, 255);
    pub const MIDNIGHT: Self = Self::new(25, 25, 112, 255);
    pub const MINT: Self = Self::new(189, 252, 201, 255);
    pub const NAVY: Self = Self::new(0, 0, 128, 255);
    pub const OLIVE: Self = Self::new(128, 128, 0, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);
    pub const ORCHID: Self = Self::new(218, 112, 214, 255);
    pub const PEACH: Self = Self::new(255, 218, 185, 255);
    pub const PINK: Self = Self::new(255, 192, 203, 255);
    pub const PURPLE: Self = Self::new(128, 0, 128, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const SALMON: Self = Self::new(250, 128, 114, 255);
    pub const TEAL: Self = Self::new(0, 128, 128, 255);
    pub const TOMATO: Self = Self::new(255, 99, 71, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const TURQUOISE: Self = Self::new(64, 224, 208, 255);
    pub const WHEAT: Self = Self::new(245, 222, 179, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_canvas_string(&self) -> JsValue {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a).into()
    }
}
