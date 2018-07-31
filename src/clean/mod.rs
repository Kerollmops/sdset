//! Operations for already deduplicated and sorted slices.

mod union;
mod intersection;
mod difference;

// FIXME allow to use #![no_std]
use std::cmp::{self, Ordering};
pub use self::union::Union;
pub use self::intersection::Intersection;
pub use self::difference::Difference;

pub struct OpBuilder<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> OpBuilder<'a, T> {
    pub fn new() -> Self {
        Self { slices: Vec::new() }
    }

    pub fn from_vec(slices: Vec<&'a [T]>) -> Self {
        Self { slices }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { slices: Vec::with_capacity(capacity) }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.slices.reserve(additional);
    }

    pub fn add(mut self, slice: &'a [T]) -> Self {
        self.push(slice);
        self
    }

    pub fn push(&mut self, slice: &'a [T]) {
        self.slices.push(slice);
    }

    pub fn union(self) -> Union<'a, T> {
        Union::new(self.slices)
    }

    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new(self.slices)
    }

    pub fn difference(self) -> Difference<'a, T> {
        Difference::new(self.slices)
    }
}

pub struct UnionTwoSlices<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: 'a> UnionTwoSlices<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        UnionTwoSlices { a, b }
    }
}

impl<'a, T: 'a + Ord + Clone> UnionTwoSlices<'a, T> {
    pub fn extend_vec(mut self, output: &mut Vec<T>) {
        let min_len = cmp::max(self.a.len(), self.b.len());
        output.reserve(min_len);

        while !self.a.is_empty() && !self.b.is_empty() {
            match self.a[0].cmp(&self.b[0]) {
                Ordering::Less => {
                    output.push(self.a[0].clone());
                    self.a = &self.a[1..];
                },
                Ordering::Equal => {
                    output.push(self.a[0].clone());
                    self.a = &self.a[1..];
                    self.b = &self.b[1..];
                },
                Ordering::Greater => {
                    output.push(self.b[0].clone());
                    self.b = &self.b[1..];
                },
            }
        }

        output.extend(self.a.iter().cloned());
        output.extend(self.b.iter().cloned());
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::new();
        self.extend_vec(&mut vec);
        vec
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

        let union: Vec<_> = UnionTwoSlices::new(a, b).into_vec();

        assert_eq!(&union, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 3];
        let b = &[];

        let union: Vec<_> = UnionTwoSlices::new(a, b).into_vec();

        assert_eq!(&union, &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 4];

        let union: Vec<_> = UnionTwoSlices::new(a, b).into_vec();

        assert_eq!(&union, &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1];
        let b = &[1];

        let union: Vec<_> = UnionTwoSlices::new(a, b).into_vec();

        assert_eq!(&union, &[1]);
    }

    #[bench]
    fn union_bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union: Vec<_> = UnionTwoSlices::new(&a, &b).into_vec();
            test::black_box(|| union);
        });
    }

    #[bench]
    fn union_bench_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_ = UnionTwoSlices::new(&a, &b).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn union_bench_two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_ = UnionTwoSlices::new(&a, &b).into_vec();
            test::black_box(|| union_);
        });
    }
}
