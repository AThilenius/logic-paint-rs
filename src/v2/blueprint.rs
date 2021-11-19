use glam::IVec2;

use super::upc::UPC;

pub struct Blueprint {
    chunks: Vec<CellChunk>,
}

struct CellChunk {
    pub offset: IVec2,
    pub cells: Vec<UPC>,
}
