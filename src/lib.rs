#![feature(test)]

extern crate test;

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

#[inline]
fn skip_duplicates<'a, T: Eq>(slice: &'a [T], elem: &'a T) -> &'a [T] {
    let count = slice.iter().take_while(|x| *x == elem).count();
    &slice[count..]
}

impl<'a, T: 'a + Ord> Iterator for UnionTwoSlices<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.a.is_empty() && !self.b.is_empty() {
            match self.a[0].cmp(&self.b[0]) {
                Ordering::Less => {
                    let ret = &self.a[0];
                    self.a = skip_duplicates(self.a, &self.a[0]);
                    return Some(ret);
                },
                Ordering::Equal => {
                    let ret = &self.a[0];
                    self.a = skip_duplicates(self.a, &self.a[0]);
                    self.b = skip_duplicates(self.b, &self.b[0]);
                    return Some(ret);
                },
                Ordering::Greater => {
                    let ret = &self.b[0];
                    self.b = skip_duplicates(self.b, &self.b[0]);
                    return Some(ret);
                },
            }
        }

        if !self.a.is_empty() {
            let ret = &self.a[0];
            self.a = skip_duplicates(self.a, &self.a[0]);
            Some(ret)
        }
        else if !self.b.is_empty() {
            let ret = &self.b[0];
            self.b = skip_duplicates(self.b, &self.b[0]);
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
    use test::Bencher;

    #[test]
    fn union_two_slices_easy() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_duplicates() {
        let a = &[1, 2, 2, 3, 3];
        let b = &[2, 3, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_duplicates_at_end() {
        let a = &[1, 2, 3, 4];
        let b = &[2, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 2, 3, 3];
        let b = &[];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();

        assert_eq!(&union, &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1, 1, 1, 1];
        let b = &[1, 1, 1, 1, 1];

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

    #[bench]
    fn bench_two_slices_duplicates(b: &mut Bencher) {
        b.iter(|| {
            let a = &[1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];
            let b = &[2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4];

            let union: Vec<_> = UnionTwoSlices::new(a, b).cloned().collect();
            test::black_box(|| union);
        });
    }
}
