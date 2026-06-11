use std::collections::HashMap;

use crate::{
    coords::ChunkCoord,
    substrate::{
        buffer::Buffer,
        mask::BufferMaskChunk,
    },
};

pub struct ChunkUploadState {
    scratch_cells: Vec<u8>,
    last_uploaded_cells: Vec<u8>,
    mask_applied: bool,
}

impl Default for ChunkUploadState {
    fn default() -> Self {
        Self {
            scratch_cells: Vec::new(),
            last_uploaded_cells: Vec::new(),
            mask_applied: false,
        }
    }
}

impl ChunkUploadState {
    pub fn upload_slice(&self) -> &[u8] {
        &self.scratch_cells
    }
}

pub fn collect_nonempty_chunks(buffer: &Buffer) -> HashMap<ChunkCoord, usize> {
    buffer
        .chunks
        .iter()
        .enumerate()
        .filter(|(_, chunk)| chunk.cell_count > 0)
        .map(|(idx, chunk)| (chunk.chunk_coord, idx))
        .collect()
}

fn pack_mask_channel(scratch: &mut [u8], mask_chunk: &BufferMaskChunk) {
    let cell_count = scratch.len() / 4;
    for cell_idx in 0..cell_count {
        let base = cell_idx * 4;
        let mut packed_mask = 0_u8;
        if mask_chunk.cells[base] > 0 {
            packed_mask |= 1 << 0;
        }
        if mask_chunk.cells[base + 1] > 0 {
            packed_mask |= 1 << 1;
        }
        if mask_chunk.cells[base + 2] > 0 {
            packed_mask |= 1 << 2;
        }
        if mask_chunk.cells[base + 3] > 0 {
            packed_mask |= 1 << 3;
        }
        scratch[base + 2] = packed_mask;
    }
}

fn clear_mask_channel(scratch: &mut [u8]) {
    let cell_count = scratch.len() / 4;
    for cell_idx in 0..cell_count {
        scratch[(cell_idx * 4) + 2] = 0;
    }
}

fn needs_texture_upload(
    cells: &[u8],
    last_uploaded: &[u8],
    mask: Option<&BufferMaskChunk>,
    mask_applied: bool,
) -> bool {
    if mask.is_some() || mask_applied {
        return true;
    }
    cells != last_uploaded
}

pub fn prepare_chunk_pixels(
    state: &mut ChunkUploadState,
    cells: &[u8],
    mask: Option<&BufferMaskChunk>,
) -> bool {
    if !needs_texture_upload(cells, &state.last_uploaded_cells, mask, state.mask_applied) {
        return false;
    }

    if state.scratch_cells.len() != cells.len() {
        state.scratch_cells.resize(cells.len(), 0);
    }
    state.scratch_cells.copy_from_slice(cells);

    if let Some(mask_chunk) = mask {
        pack_mask_channel(&mut state.scratch_cells, mask_chunk);
        state.mask_applied = true;
    } else if state.mask_applied {
        clear_mask_channel(&mut state.scratch_cells);
        state.mask_applied = false;
    }

    if state.scratch_cells.as_slice() == state.last_uploaded_cells.as_slice() {
        return false;
    }

    if state.last_uploaded_cells.len() != state.scratch_cells.len() {
        state
            .last_uploaded_cells
            .resize(state.scratch_cells.len(), 0);
    }
    state
        .last_uploaded_cells
        .copy_from_slice(state.scratch_cells.as_slice());
    true
}
