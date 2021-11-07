use glam::IVec2;

use super::upc::UPC;

// Blueprint: Serialized cell chunks which include modules who's root resides in that chunk, as well
// data blobs referenced by modules (ex. RAM data or JS source). Cells are packed into dense lists
// of u32: [c_x: u8, c_y: u8, flags: u16]. Modules are bincode serialized. Blob data is serialized
// in any format. All blueprints are stored as binary data that is gziped, then base64 UTF-8
// encoded, with the preamble "LPBPV1[<NAME>]:"

pub struct Blueprint {
    chunks: Vec<CellChunk>,
}

struct CellChunk {
    pub offset: IVec2,
    pub cells: Vec<UPC>,
}
