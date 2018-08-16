use std::cmp::{self, Ordering};
use set::Set;
use ::SetOperation;

/// Represent the _union_ set operation that will be applied to two slices.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::duo::OpBuilder;
/// use sdset::{SetOperation, Set};
///
/// let a = Set::new(&[1, 2, 4, 6, 7])?;
/// let b = Set::new(&[2, 3, 4, 5, 6, 7])?;
///
/// let op = OpBuilder::new(a, b).union();
///
/// let res = op.into_set_buf();
/// assert_eq!(&res[..], &[1, 2, 3, 4, 5, 6, 7]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct Union<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: 'a> Union<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self::new_unchecked(a.as_slice(), b.as_slice())
    }

    /// Construct one with unchecked slices.
    pub fn new_unchecked(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b }
    }
}

impl<'a, T: Ord + Clone> SetOperation<&'a T, T> for Union<'a, T> {
    fn extend_vec(mut self, output: &mut Vec<T>) {
        let min_len = cmp::max(self.a.len(), self.b.len());
        output.reserve(min_len);

        while !self.a.is_empty() && !self.b.is_empty() {
            let a = &self.a[0];
            let b = &self.b[0];

            match a.cmp(&b) {
                 Ordering::Less => {
                    let off = self.a.iter().take_while(|&x| x < b).count();
                    output.extend_from_slice(&self.a[..off]);

                    self.a = &self.a[off..];
                 },
                 Ordering::Equal => {
                    let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                    output.extend_from_slice(&self.a[..off]);

                    self.a = &self.a[off..];
                    self.b = &self.b[off..];
                 },
                 Ordering::Greater => {
                    let off = self.b.iter().take_while(|&x| x < a).count();
                    output.extend_from_slice(&self.b[..off]);

                    self.b = &self.b[off..];
                 },
             }
        }

        output.extend_from_slice(self.a);
        output.extend_from_slice(self.b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_two_slices_easy() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union_ = Union::new_unchecked(a, b).into_set_buf();

        assert_eq!(&union_[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 3];
        let b = &[];

        let union_ = Union::new_unchecked(a, b).into_set_buf();

        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 4];

        let union_ = Union::new_unchecked(a, b).into_set_buf();

        assert_eq!(&union_[..], &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1];
        let b = &[1];

        let union_ = Union::new_unchecked(a, b).into_set_buf();

        assert_eq!(&union_[..], &[1]);
    }

    quickcheck! {
        fn qc_union(a: Vec<i32>, b: Vec<i32>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;

            ::sort_dedup_vec(&mut a);
            ::sort_dedup_vec(&mut b);

            let x = Union::new_unchecked(&a, &b).into_set_buf();

            let a = BTreeSet::from_iter(a);
            let b = BTreeSet::from_iter(b);
            let y = a.union(&b);
            let y: Vec<_> = y.cloned().collect();

            x.as_slice() == y.as_slice()
        }
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use super::*;
    use self::test::Bencher;

    #[bench]
    fn two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(&a, &b).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(&a, &b).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(&a, &b).into_set_buf();
            test::black_box(|| union_);
        });
    }
}
