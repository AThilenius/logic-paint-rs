pub struct RangeIterator {
    i: i32,
    end: i32,
    step: i32,
}

impl Iterator for RangeIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.end {
            return None;
        }

        let i = self.i;
        self.i += self.step;

        Some(i)
    }
}

pub fn range_iter(from: i32, to: i32) -> RangeIterator {
    RangeIterator {
        i: from,
        end: to,
        step: if from < to { 1 } else { -1 },
    }
}
