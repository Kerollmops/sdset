use std::cmp::Ordering;
use crate::set::Set;
use crate::{SetOperation, Collection};

/// Represent the _symmetric difference_ set operation that will be applied to two slices.
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
/// let op = OpBuilder::new(a, b).symmetric_difference();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1, 3, 5]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct SymmetricDifference<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> SymmetricDifference<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self {
            a: a.as_slice(),
            b: b.as_slice(),
        }
    }
}

impl<'a, T: Ord> SymmetricDifference<'a, T> {
    #[inline]
    fn extend_collection<C, U, F>(mut self, output: &mut C, extend: F) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
    {
        loop {
            match (self.a.first(), self.b.first()) {
                (Some(a), Some(b)) => {
                    match a.cmp(b) {
                        Ordering::Less => {
                            let off = self.a.iter().take_while(|&e| e < b).count();
                            extend(output, &self.a[..off])?;
                            self.a = &self.a[off..];
                        },
                        Ordering::Equal => {
                            let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                            self.a = &self.a[off..];
                            self.b = &self.b[off..];
                        },
                        Ordering::Greater => {
                            let off = self.b.iter().take_while(|&e| e < a).count();
                            extend(output, &self.b[..off])?;
                            self.b = &self.b[off..];
                        },
                    }
                },
                (Some(_), None) => {
                    extend(output, self.a)?;
                    break;
                },
                (None, Some(_)) => {
                    extend(output, self.b)?;
                    break;
                },
                (None, None) => break,
            }
        }
        Ok(())
    }

    fn iter(&self) -> SymmetricDifferenceIter<'a, T>
    {
        SymmetricDifferenceIter {
            a: self.a,
            b: self.b
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for SymmetricDifference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for SymmetricDifference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend)
    }
}

impl<'a, T: Ord> IntoIterator for SymmetricDifference<'a, T> {
    type Item = &'a T;
    type IntoIter = SymmetricDifferenceIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Ord> IntoIterator for &'a SymmetricDifference<'a, T> {
    type Item = &'a T;
    type IntoIter = SymmetricDifferenceIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct SymmetricDifferenceIter<'a, T> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: Ord> Iterator for SymmetricDifferenceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.a.first(), self.b.first()) {
                (Some(first_a), Some(first_b)) => {
                    match first_a.cmp(first_b) {
                        Ordering::Less => {
                            self.a = &self.a[1..];
                            return Some(first_a);
                        },
                        Ordering::Equal => {
                            let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                            self.a = &self.a[off..];
                            self.b = &self.b[off..];
                        },
                        Ordering::Greater => {
                            self.b = &self.b[1..];
                            return Some(first_b);
                        },
                    }
                },
                (Some(first_a), None) => {
                    self.a = &self.a[1..];
                    return Some(first_a);
                },
                (None, Some(first_b)) => {
                    self.b = &self.b[1..];
                    return Some(first_b);
                },
                (None, None) => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod set_to_set {
        use super::super::*;
        use crate::set::{sort_dedup_vec, SetBuf};

        quickcheck! {
            fn qc_symmetric_difference(a: Vec<i32>, b: Vec<i32>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut a = a;
                let mut b = b;

                sort_dedup_vec(&mut a);
                sort_dedup_vec(&mut b);

                let x: SetBuf<i32> = SymmetricDifference { a: &a, b: &b }.into_set_buf();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let y = a.symmetric_difference(&b);
                let y: Vec<_> = y.cloned().collect();

                x.as_slice() == y.as_slice()
            }
        }
    }
    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        quickcheck! {
            fn qc_symmetric_difference(a: Vec<i32>, b: Vec<i32>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut a = a;
                let mut b = b;

                sort_dedup_vec(&mut a);
                sort_dedup_vec(&mut b);

                let x: Vec<i32> = SymmetricDifference { a: &a, b: &b }.into_iter().cloned().collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let y = a.symmetric_difference(&b);
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
            let symmetric_difference_: SetBuf<i32> = SymmetricDifference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| symmetric_difference_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let symmetric_difference_: SetBuf<i32> = SymmetricDifference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| symmetric_difference_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let symmetric_difference_: SetBuf<i32> = SymmetricDifference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| symmetric_difference_);
        });
    }
}
