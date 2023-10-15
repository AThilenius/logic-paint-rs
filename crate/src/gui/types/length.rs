use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Length {
    // Size is determined by the sum of the desired sizes of its children.
    Auto,

    // Size will always be exactly this value, even if it results in overflow.
    Pixels(f32),

    // Size is determined by summing all weighted children, and dividing the non-fixed/auto
    // remaining space weighted among them. A single weighted child results in a "Full" size. A
    // weighted size used on a minor axis (ie. not the axis being layed out) is treated as "Auto".
    Weighted(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Auto
    }
}
