//! Operations for already deduplicated and sorted slices.

mod union;

// FIXME allow to use #![no_std]
use std::cmp::{self, Ordering};
pub use self::union::Union;

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
    pub fn into_vec(mut self) -> Vec<T> {
        let min_len = cmp::max(self.a.len(), self.b.len());
        let mut output = Vec::with_capacity(min_len);

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

        output
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
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union: Vec<_> = UnionTwoSlices::new(&a, &b).into_vec();
            test::black_box(|| union);
        });
    }
}
