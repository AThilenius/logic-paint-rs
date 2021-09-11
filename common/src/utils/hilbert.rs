use bevy::math::{IVec2, UVec2};
use fast_hilbert::{h2xy, xy2h};
use shrinkwraprs::Shrinkwrap;

#[derive(Shrinkwrap, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HilbertCode(pub u64);

pub struct HilbertArray<T> {
    pub size: usize,
    pub raw_data: Vec<T>,
}

impl<T> HilbertArray<T>
where
    T: Default + Clone,
{
    pub fn new_2d(size: usize) -> Self {
        Self {
            size,
            raw_data: vec![T::default(); size * size],
        }
    }
}

pub trait HilbertIndexing<T, H> {
    fn get(&self, h: H) -> &T;
    fn get_mut(&mut self, h: H) -> &mut T;
    fn get_checked(&self, h: H) -> Option<&T>;
    fn get_mut_checked(&mut self, h: H) -> Option<&mut T>;
}

impl<T, H> HilbertIndexing<T, H> for HilbertArray<T>
where
    H: Into<HilbertCode>,
{
    #[inline(always)]
    fn get(&self, h: H) -> &T {
        let code: HilbertCode = h.into();
        &self.raw_data[code.0 as usize]
    }

    #[inline(always)]
    fn get_mut(&mut self, h: H) -> &mut T {
        let code: HilbertCode = h.into();
        &mut self.raw_data[code.0 as usize]
    }

    #[inline(always)]
    fn get_checked(&self, h: H) -> Option<&T> {
        let code: HilbertCode = h.into();
        if code.0 as usize >= self.raw_data.len() {
            None
        } else {
            Some(&self.raw_data[code.0 as usize])
        }
    }

    #[inline(always)]
    fn get_mut_checked(&mut self, h: H) -> Option<&mut T> {
        let code: HilbertCode = h.into();
        if code.0 as usize >= self.raw_data.len() {
            None
        } else {
            Some(&mut self.raw_data[code.0 as usize])
        }
    }
}

impl From<(i32, i32)> for HilbertCode {
    #[inline(always)]
    fn from(v: (i32, i32)) -> Self {
        Self(xy2h(v.0 as u32, v.1 as u32))
    }
}

impl From<HilbertCode> for (i32, i32) {
    #[inline(always)]
    fn from(h: HilbertCode) -> Self {
        let (x, y): (u32, u32) = h2xy(h.0);
        (x as i32, y as i32)
    }
}

impl From<(u32, u32)> for HilbertCode {
    #[inline(always)]
    fn from(v: (u32, u32)) -> Self {
        Self(xy2h(v.0, v.1))
    }
}

impl From<HilbertCode> for (u32, u32) {
    #[inline(always)]
    fn from(h: HilbertCode) -> Self {
        let (x, y) = h2xy(h.0);
        (x, y)
    }
}

impl From<(usize, usize)> for HilbertCode {
    #[inline(always)]
    fn from(v: (usize, usize)) -> Self {
        Self(xy2h(v.0 as u32, v.1 as u32))
    }
}

impl From<HilbertCode> for (usize, usize) {
    #[inline(always)]
    fn from(h: HilbertCode) -> Self {
        let (x, y): (u32, u32) = h2xy(h.0);
        (x as usize, y as usize)
    }
}

impl From<IVec2> for HilbertCode {
    #[inline(always)]
    fn from(v: IVec2) -> Self {
        Self(xy2h(v.x as u32, v.y as u32))
    }
}

impl From<HilbertCode> for IVec2 {
    #[inline(always)]
    fn from(h: HilbertCode) -> Self {
        let (x, y): (u32, u32) = h2xy(h.0);
        Self::new(x as i32, y as i32)
    }
}

impl From<UVec2> for HilbertCode {
    #[inline(always)]
    fn from(v: UVec2) -> Self {
        Self(xy2h(v.x, v.y))
    }
}

impl From<HilbertCode> for UVec2 {
    #[inline(always)]
    fn from(h: HilbertCode) -> Self {
        let (x, y) = h2xy(h.0);
        Self::new(x, y)
    }
}

impl From<usize> for HilbertCode {
    #[inline(always)]
    fn from(h: usize) -> Self {
        Self(h as u64)
    }
}

impl Into<usize> for HilbertCode {
    #[inline(always)]
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<u64> for HilbertCode {
    #[inline(always)]
    fn from(h: u64) -> Self {
        Self(h as u64)
    }
}

impl Into<u64> for HilbertCode {
    #[inline(always)]
    fn into(self) -> u64 {
        self.0
    }
}

impl From<u32> for HilbertCode {
    #[inline(always)]
    fn from(h: u32) -> Self {
        Self(h as u64)
    }
}

impl Into<u32> for HilbertCode {
    #[inline(always)]
    fn into(self) -> u32 {
        self.0 as u32
    }
}
