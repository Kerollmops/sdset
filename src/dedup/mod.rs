//! Operations for already deduplicated slices.

// FIXME allow to use #![no_std]
use std::cmp::Ordering;

pub struct UnionTwoSlices<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: 'a> UnionTwoSlices<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        UnionTwoSlices { a, b }
    }
}

impl<'a, T: 'a + Ord> Iterator for UnionTwoSlices<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.a.is_empty() && !self.b.is_empty() {
            match self.a[0].cmp(&self.b[0]) {
                Ordering::Less => {
                    let ret = &self.a[0];
                    self.a = &self.a[1..];
                    return Some(ret);
                },
                Ordering::Equal => {
                    let ret = &self.a[0];
                    self.a = &self.a[1..];
                    self.b = &self.b[1..];
                    return Some(ret);
                },
                Ordering::Greater => {
                    let ret = &self.b[0];
                    self.b = &self.b[1..];
                    return Some(ret);
                },
            }
        }

        if !self.a.is_empty() {
            let ret = &self.a[0];
            self.a = &self.a[1..];
            Some(ret)
        }
        else if !self.b.is_empty() {
            let ret = &self.b[0];
            self.b = &self.b[1..];
            Some(ret)
        }
        else {
            None
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::{self, Bencher};

    #[test]
    fn union_two_slices_easy() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 3];
        let b = &[];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1];
        let b = &[1];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1]);
    }

    #[bench]
    fn bench_two_slices_easy(b: &mut Bencher) {
        b.iter(|| {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();
            test::black_box(|| union);
        });
    }
}
