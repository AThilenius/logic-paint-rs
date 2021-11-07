use glam::{IVec2, UVec2};

pub(crate) const CHUNK_SIZE: usize = 32;
pub(crate) const LOG_CHUNK_SIZE: usize = 5;
const UPPER_MASK: i32 = !((CHUNK_SIZE as i32) - 1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CellCoord(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkCoord(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocalCoord(pub UVec2);

impl From<(i32, i32)> for CellCoord {
    fn from(v: (i32, i32)) -> Self {
        Self(IVec2::new(v.0, v.1))
    }
}

impl From<CellCoord> for ChunkCoord {
    #[inline(always)]
    fn from(c: CellCoord) -> Self {
        Self(IVec2::new(c.0.x >> LOG_CHUNK_SIZE, c.0.y >> LOG_CHUNK_SIZE))
    }
}

impl From<&CellCoord> for ChunkCoord {
    #[inline(always)]
    fn from(c: &CellCoord) -> Self {
        Self(IVec2::new(c.0.x >> LOG_CHUNK_SIZE, c.0.y >> LOG_CHUNK_SIZE))
    }
}

impl From<CellCoord> for LocalCoord {
    #[inline(always)]
    fn from(c: CellCoord) -> Self {
        Self(UVec2::new(
            (c.0.x - (c.0.x & UPPER_MASK)) as u32,
            (c.0.y - (c.0.y & UPPER_MASK)) as u32,
        ))
    }
}

impl From<&CellCoord> for LocalCoord {
    #[inline(always)]
    fn from(c: &CellCoord) -> Self {
        Self(UVec2::new(
            (c.0.x - (c.0.x & UPPER_MASK)) as u32,
            (c.0.y - (c.0.y & UPPER_MASK)) as u32,
        ))
    }
}

impl ChunkCoord {
    #[inline(always)]
    pub fn first_cell_coord(&self) -> CellCoord {
        CellCoord(IVec2::new(
            self.0.x << LOG_CHUNK_SIZE,
            self.0.y << LOG_CHUNK_SIZE,
        ))
    }
}

impl LocalCoord {
    #[inline(always)]
    pub fn to_cell_coord(&self, chunk_coord: &ChunkCoord) -> CellCoord {
        CellCoord(self.0.as_ivec2() + chunk_coord.first_cell_coord().0)
    }
}
