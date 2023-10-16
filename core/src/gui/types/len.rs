use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Len {
    // Size is determined by the sum of the desired sizes of its children.
    Auto,

    // Size will always be exactly this value, even if it results in overflow.
    Pixels(f32),

    // Size is determined by summing all weighted children, and dividing the non-fixed/auto
    // remaining space weighted among them. A single weighted child results in a "Full" size. A
    // weighted size used on a minor axis (ie. not the axis being layed out) is treated as "Auto".
    Weighted(f32),
}

pub trait ToPixels {
    fn pixels(self) -> Len;
}

impl ToPixels for i32 {
    fn pixels(self) -> Len {
        Len::Pixels(self as f32)
    }
}

pub trait ToWeighted {
    fn weighted(self) -> Len;
}

impl ToWeighted for i32 {
    fn weighted(self) -> Len {
        Len::Weighted(self as f32)
    }
}

impl Default for Len {
    fn default() -> Self {
        Self::Auto
    }
}
