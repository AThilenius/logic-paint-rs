use serde::{Deserialize, Serialize};

use super::{box_size::BoxSize, Color};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Border {
    pub size: BoxSize,
    pub color: Color,
}
