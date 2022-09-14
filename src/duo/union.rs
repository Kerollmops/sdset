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
    fn extend_collection<C, U, F>(mut self, output: &mut C, extend: F) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
    {
        let min_len = cmp::max(self.a.len(), self.b.len());
        output.reserve(min_len)?;

        while !self.a.is_empty() && !self.b.is_empty() {
            let first_a = &self.a[0];
            let first_b = &self.b[0];

            match first_a.cmp(&first_b) {
                 Ordering::Less => {
                    let off = self.a.iter().take_while(|&x| x < first_b).count();
                    extend(output, &self.a[..off])?;

                    self.a = &self.a[off..];
                 },
                 Ordering::Equal => {
                    let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                    extend(output, &self.a[..off])?;

                    self.a = &self.a[off..];
                    self.b = &self.b[off..];
                 },
                 Ordering::Greater => {
                    let off = self.b.iter().take_while(|&x| x < first_a).count();
                    extend(output, &self.b[..off])?;

                    self.b = &self.b[off..];
                 },
             }
        }

        extend(output, self.a)?;
        extend(output, self.b)?;
        Ok(())
    }

    fn iter(&self) -> UnionIter<'a, T>
    {
        UnionIter {
            a: self.a,
            b: self.b
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend)
    }
}

impl<'a, T: Ord> IntoIterator for Union<'a, T> {
    type Item = &'a T;
    type IntoIter = UnionIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Ord> IntoIterator for &'a Union<'a, T> {
    type Item = &'a T;
    type IntoIter = UnionIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct UnionIter<'a, T> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: Ord> Iterator for UnionIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.a.is_empty() {
            let result = self.b.first();
            if result.is_some() {
                self.b = &self.b[1..];
            }
            return result;
        }
        if self.b.is_empty() {
            let result = self.a.first();
            if result.is_some() {
                self.a = &self.a[1..];
            }
            return result;
        }
        let first_a = &self.a[0];
        let first_b = &self.b[0];

        match first_a.cmp(&first_b) {
            Ordering::Less => {
                self.a = &self.a[1..];
                return Some(first_a);
            },
            Ordering::Equal => {
                self.a = &self.a[1..];
                self.b = &self.b[1..];
                return Some(first_a);
            },
            Ordering::Greater => {
                self.b = &self.b[1..];
                return Some(first_b);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    mod set_to_set {
        use super::super::*;
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

    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        #[test]
        fn union_two_slices_easy() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let union_: Vec<i32> = Union { a: a, b: b }.into_iter().cloned().collect();

            assert_eq!(&union_[..], &[1, 2, 3, 4]);
        }

        #[test]
        fn union_two_slices_second_empty() {
            let a = &[1, 2, 3];
            let b = &[];

            let union_: Vec<i32> = Union { a: a, b: b }.into_iter().cloned().collect();

            assert_eq!(&union_[..], &[1, 2, 3]);
        }

        #[test]
        fn union_two_slices_first_empty() {
            let a = &[];
            let b = &[2, 3, 4];

            let union_: Vec<i32> = Union { a: a, b: b }.into_iter().cloned().collect();

            assert_eq!(&union_[..], &[2, 3, 4]);
        }

        #[test]
        fn union_two_slices_same_elem() {
            let a = &[1];
            let b = &[1];

            let union_: Vec<i32> = Union { a: a, b: b }.into_iter().cloned().collect();

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

                let x: Vec<i32> = Union { a: &a, b: &b }.into_iter().cloned().collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let y = a.union(&b);
                let y: Vec<_> = y.cloned().collect();

                x.as_slice() == y.as_slice()
            }
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
