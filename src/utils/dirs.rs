use glam::IVec2;
use smallvec::{SmallVec, smallvec};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Dirs {
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

impl From<IVec2> for Dirs {
    #[inline(always)]
    fn from(v: IVec2) -> Self {
        Self {
            up: v == IVec2::Y,
            right: v == IVec2::X,
            down: v == -IVec2::Y,
            left: v == -IVec2::X,
        }
    }
}

impl Dirs {
    pub fn get_offsets(&self) -> SmallVec<[IVec2; 4]> {
        let mut vec = smallvec![];

        if self.up {
            vec.push(IVec2::Y)
        }
        if self.right {
            vec.push(IVec2::X)
        }
        if self.down {
            vec.push(-IVec2::Y)
        }
        if self.left {
            vec.push(-IVec2::X)
        }

        vec
    }

    pub fn set_direction(&mut self, dir: IVec2, value: bool) {
        // Note y coords are inverted.
        match (dir.x, dir.y) {
            (0, 1) => self.up = value,
            (1, 0) => self.right = value,
            (0, -1) => self.down = value,
            (-1, 0) => self.left = value,
            _ => panic!("Unsupported vector for set_direction"),
        }
    }
}
