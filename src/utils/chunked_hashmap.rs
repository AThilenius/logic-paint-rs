use std::collections::HashMap;

use glam::IVec2;

use crate::wgl2::LOG_CHUNK_SIZE;

#[derive(Default)]
pub struct ChunkedHashMap<T>
where
    T: PartialEq + Default,
{
    chunks: Vec<HashMap<IVec2, T>>,
    chunk_lookup_by_chunk_idx: HashMap<IVec2, usize>,
}

impl<T> ChunkedHashMap<T>
where
    T: PartialEq + Default,
{
    #[inline]
    pub fn get_cell(&self, cell_loc: &IVec2) -> Option<&T> {
        let chunk_loc = IVec2::new(cell_loc.x >> LOG_CHUNK_SIZE, cell_loc.y >> LOG_CHUNK_SIZE);

        if let Some(chunk_idx) = self.chunk_lookup_by_chunk_idx.get(&chunk_loc) {
            return self.chunks[*chunk_idx].get(cell_loc);
        }

        None
    }

    #[inline]
    pub fn get_chunk(&self, chunk_loc: &IVec2) -> Option<&HashMap<IVec2, T>> {
        if let Some(chunk_idx) = self.chunk_lookup_by_chunk_idx.get(&chunk_loc) {
            return Some(&self.chunks[*chunk_idx]);
        }

        None
    }

    #[inline]
    pub fn set_cell(&mut self, cell_loc: IVec2, cell: T) {
        let chunk_loc = IVec2::new(cell_loc.x >> LOG_CHUNK_SIZE, cell_loc.y >> LOG_CHUNK_SIZE);

        if cell == Default::default() {
            // Delete op.
            if let Some(&chunk_idx) = self.chunk_lookup_by_chunk_idx.get(&chunk_loc) {
                let chunk = &mut self.chunks[chunk_idx];
                chunk.remove(&cell_loc);

                if chunk.len() == 0 {
                    self.chunks.remove(chunk_idx);
                    self.chunk_lookup_by_chunk_idx.remove(&chunk_loc);
                }
            }
        } else {
            // Set op. Need to create a chunk if it doesn't already exist.
            let chunk_idx = if let Some(chunk_idx) = self.chunk_lookup_by_chunk_idx.get(&chunk_loc)
            {
                *chunk_idx
            } else {
                let idx = self.chunks.len();
                self.chunk_lookup_by_chunk_idx.insert(chunk_loc, idx);
                self.chunks.push(Default::default());
                idx
            };

            self.chunks[chunk_idx].insert(cell_loc, cell);
        }
    }
}
