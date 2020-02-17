use std::cmp::{self, Ordering};
use crate::set::Set;
use crate::{SetOperation, Collection};

/// Represent the _union_ set operation that will be applied to two slices.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::duo::OpBuilder;
/// use sdset::{SetOperation, Set, SetBuf};
///
/// let a = Set::new(&[1, 2, 4, 6, 7])?;
/// let b = Set::new(&[2, 3, 4, 5, 6, 7])?;
///
/// let op = OpBuilder::new(a, b).union();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1, 2, 3, 4, 5, 6, 7]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct Union<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> Union<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self {
            a: a.as_slice(),
            b: b.as_slice(),
        }
    }
}

impl<'a, T: Ord> Union<'a, T> {
    #[inline]
    fn extend_collection<C, U, F>(mut self, output: &mut C, extend: F)
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T])
    {
        let min_len = cmp::max(self.a.len(), self.b.len());
        output.reserve(min_len);

        while !self.a.is_empty() && !self.b.is_empty() {
            let a = &self.a[0];
            let b = &self.b[0];

            match a.cmp(&b) {
                 Ordering::Less => {
                    let off = self.a.iter().take_while(|&x| x < b).count();
                    extend(output, &self.a[..off]);

                    self.a = &self.a[off..];
                 },
                 Ordering::Equal => {
                    let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                    extend(output, &self.a[..off]);

                    self.a = &self.a[off..];
                    self.b = &self.b[off..];
                 },
                 Ordering::Greater => {
                    let off = self.b.iter().take_while(|&x| x < a).count();
                    extend(output, &self.b[..off]);

                    self.b = &self.b[off..];
                 },
             }
        }

        extend(output, self.a);
        extend(output, self.b);
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) where C: Collection<T> {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) where C: Collection<&'a T> {
        self.extend_collection(output, Collection::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    #[test]
    fn union_two_slices_easy() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union_: SetBuf<i32> = Union { a: a, b: b }.into_set_buf();

        assert_eq!(&union_[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 3];
        let b = &[];

        let union_: SetBuf<i32> = Union { a: a, b: b }.into_set_buf();

        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 4];

        let union_: SetBuf<i32> = Union { a: a, b: b }.into_set_buf();

        assert_eq!(&union_[..], &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1];
        let b = &[1];

        let union_: SetBuf<i32> = Union { a: a, b: b }.into_set_buf();

        assert_eq!(&union_[..], &[1]);
    }

    quickcheck! {
        fn qc_union(a: Vec<i32>, b: Vec<i32>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;

            sort_dedup_vec(&mut a);
            sort_dedup_vec(&mut b);

            let x: SetBuf<i32> = Union { a: &a, b: &b }.into_set_buf();

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
    use crate::set::SetBuf;

    #[bench]
    fn two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { a: &a, b: &b }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { a: &a, b: &b }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { a: &a, b: &b }.into_set_buf();
            test::black_box(|| union_);
        });
    }
}
