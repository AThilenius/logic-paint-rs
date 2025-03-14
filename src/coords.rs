use glam::{IVec2, UVec2};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::prelude::*;

use crate::upc::LOG_UPC_BYTE_LEN;

pub(crate) const LOG_CHUNK_SIZE: usize = 7;
pub(crate) const CHUNK_SIZE: usize = 1 << LOG_CHUNK_SIZE;
pub(crate) const CHUNK_CELL_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE;
const UPPER_MASK: i32 = !((CHUNK_SIZE as i32) - 1);
const LOWER_MASK: usize = CHUNK_SIZE - 1;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[wasm_bindgen]
pub struct CellCoord(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkCoord(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LocalCoord(pub UVec2);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CellCoordOffset(pub IVec2);

#[derive(Serialize, Deserialize)]
pub enum Coord {
    Cell(IVec2),
    Chunk(IVec2),
    Local(UVec2),
}

impl Serialize for CellCoord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.0.x, self.0.y))
    }
}

impl<'de> Deserialize<'de> for CellCoord {
    fn deserialize<D>(deserializer: D) -> Result<CellCoord, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        let splits: Vec<_> = str.split(":").collect();

        if splits.len() != 2 {
            return Err(serde::de::Error::custom(
                "Invalid CellCoord format, expected 123:123",
            ));
        }

        let x = splits[0].parse::<i32>().map_err(serde::de::Error::custom)?;
        let y = splits[1].parse::<i32>().map_err(serde::de::Error::custom)?;

        Ok(Self(IVec2::new(x, y)))
    }
}

impl Serialize for ChunkCoord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.0.x, self.0.y))
    }
}

impl<'de> Deserialize<'de> for ChunkCoord {
    fn deserialize<D>(deserializer: D) -> Result<ChunkCoord, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        let splits: Vec<_> = str.split(":").collect();

        if splits.len() != 2 {
            return Err(serde::de::Error::custom(
                "Invalid CellCoord format, expected 123:123",
            ));
        }

        let x = splits[0].parse::<i32>().map_err(serde::de::Error::custom)?;
        let y = splits[1].parse::<i32>().map_err(serde::de::Error::custom)?;

        Ok(Self(IVec2::new(x, y)))
    }
}

impl From<CellCoord> for u64 {
    fn from(cell_coord: CellCoord) -> Self {
        (cell_coord.0.y as u64) << 32 | (cell_coord.0.x as u64)
    }
}

impl From<(i32, i32)> for CellCoord {
    fn from(v: (i32, i32)) -> Self {
        Self(IVec2::new(v.0, v.1))
    }
}

impl From<(i32, i32)> for CellCoordOffset {
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

impl CellCoord {
    #[inline(always)]
    pub fn from_offset_into_chunk(chunk_coord: &ChunkCoord, x: usize, y: usize) -> Self {
        CellCoord(IVec2::new(
            (chunk_coord.0.x << LOG_CHUNK_SIZE) + x as i32,
            (chunk_coord.0.y << LOG_CHUNK_SIZE) + y as i32,
        ))
    }
}

#[wasm_bindgen]
impl CellCoord {
    #[wasm_bindgen(constructor)]
    pub fn _wasm_ctor(x: i32, y: i32) -> Self {
        (x, y).into()
    }
}

#[allow(dead_code)]
impl ChunkCoord {
    pub fn first_cell_coord(&self) -> CellCoord {
        CellCoord(IVec2::new(
            self.0.x << LOG_CHUNK_SIZE,
            self.0.y << LOG_CHUNK_SIZE,
        ))
    }

    pub fn last_cell_coord(&self) -> CellCoord {
        CellCoord(IVec2::new(
            (self.0.x << LOG_CHUNK_SIZE) + CHUNK_SIZE as i32 - 1,
            (self.0.y << LOG_CHUNK_SIZE) + CHUNK_SIZE as i32 - 1,
        ))
    }
}

#[allow(dead_code)]
impl LocalCoord {
    #[inline(always)]
    pub fn to_cell_coord(&self, chunk_coord: &ChunkCoord) -> CellCoord {
        CellCoord(self.0.as_ivec2() + chunk_coord.first_cell_coord().0)
    }

    #[inline(always)]
    pub fn from_upc_idx(mut idx: usize) -> LocalCoord {
        idx = idx >> LOG_UPC_BYTE_LEN;
        let y = idx >> LOG_CHUNK_SIZE;
        let x = idx & LOWER_MASK;

        LocalCoord(UVec2::new(x as u32, y as u32))
    }

    #[inline(always)]
    pub fn to_upc_idx(&self) -> usize {
        (((self.0.y as usize) << LOG_CHUNK_SIZE) | self.0.x as usize) << LOG_UPC_BYTE_LEN
    }
}

impl CellCoordOffset {
    pub fn to_cell_coord(&self, anchor: CellCoord) -> CellCoord {
        CellCoord(anchor.0 + self.0)
    }
}

impl From<CellCoordOffset> for u64 {
    fn from(offset: CellCoordOffset) -> Self {
        (offset.0.y as u64) << 32 | (offset.0.x as u64)
    }
}
