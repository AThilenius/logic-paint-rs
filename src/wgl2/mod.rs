use glam::IVec2;

pub use camera::Camera;
pub use cell_program::CellProgram;
pub use cell_render_chunk::CellRenderChunk;
pub use quad_vao::QuadVao;
pub use uniform::*;

mod camera;
mod cell_program;
mod cell_render_chunk;
mod quad_vao;
mod uniform;

pub const CHUNK_SIZE: usize = 32;
pub const LOG_CHUNK_SIZE: usize = 5;

#[inline(always)]
pub fn cell_to_chunk_loc(loc: &IVec2) -> IVec2 {
    // Right shift LOG(CHUNK_SIZE) bits, which is: divide by 32, with a floor op.
    IVec2::new(loc.x >> LOG_CHUNK_SIZE, loc.y >> LOG_CHUNK_SIZE)
}
