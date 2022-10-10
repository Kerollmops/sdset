use crate::set::Set;
use crate::{exponential_offset_ge, SetOperation, Collection};

/// Represent the _intersection_ set operation that will be applied to two slices.
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
/// let op = OpBuilder::new(a, b).intersection();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[2, 4, 6, 7]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct Intersection<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> Intersection<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self {
            a: a.as_slice(),
            b: b.as_slice(),
        }
    }
}

impl<'a, T: Ord> Intersection<'a, T> {
    #[inline]
    fn extend_collection<C, U, F>(mut self, output: &mut C, extend: F) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
    {
        while !self.a.is_empty() && !self.b.is_empty() {
            let first_a = &self.a[0];
            let first_b = &self.b[0];

            if first_a == first_b {
                let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                extend(output, &self.a[..off])?;

                self.a = &self.a[off..];
                self.b = &self.b[off..];
            }
            else if first_a < first_b {
                self.a = exponential_offset_ge(self.a, first_b);
            }
            else {
                self.b = exponential_offset_ge(self.b, first_a);
            }
        }
        Ok(())
    }

    fn iter(&self) -> IntersectionIter<'a, T>
    {
        IntersectionIter {
            a: self.a,
            b: self.b
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Intersection<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Intersection<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend)
    }
}

impl<'a, T: Ord> IntoIterator for Intersection<'a, T> {
    type Item = &'a T;
    type IntoIter = IntersectionIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Ord> IntoIterator for &'a Intersection<'a, T> {
    type Item = &'a T;
    type IntoIter = IntersectionIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntersectionIter<'a, T> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: Ord> Iterator for IntersectionIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.a.is_empty() || self.b.is_empty() {
                return None;
            }
            let first_a = &self.a[0];
            let first_b = &self.b[0];
            if first_a == first_b {
                self.a = &self.a[1..];
                self.b = &self.b[1..];
                return Some(first_a);
            } else if first_a < first_b {
                self.a = exponential_offset_ge(self.a, first_b);
            } else {
                self.b = exponential_offset_ge(self.b, first_a);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod set_to_set {
        use super::super::*;
        use crate::set::{sort_dedup_vec, SetBuf};

        #[test]
        fn two_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let intersection_: SetBuf<i32> = Intersection { a: a, b: b }.into_set_buf();
            assert_eq!(&intersection_[..], &[2, 3]);
        }

        quickcheck! {
            fn qc_intersection(a: Vec<i32>, b: Vec<i32>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut a = a;
                let mut b = b;

                sort_dedup_vec(&mut a);
                sort_dedup_vec(&mut b);

                let x: SetBuf<i32> = Intersection { a: &a, b: &b }.into_set_buf();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let y = a.intersection(&b);
                let y: Vec<_> = y.cloned().collect();

                x.as_slice() == y.as_slice()
            }
        }
    }
    
    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        #[test]
        fn two_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let intersection_: Vec<i32> = Intersection { a: a, b: b }.into_iter().cloned().collect();
            assert_eq!(&intersection_[..], &[2, 3]);
        }

        quickcheck! {
            fn qc_intersection(a: Vec<i32>, b: Vec<i32>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut a = a;
                let mut b = b;

                sort_dedup_vec(&mut a);
                sort_dedup_vec(&mut b);

                let x: Vec<i32> = Intersection { a: &a, b: &b }.into_iter().cloned().collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let y = a.intersection(&b);
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
            let intersection_: SetBuf<i32> = Intersection { a: &a, b: &b }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { a: &a, b: &b }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { a: &a, b: &b }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }
}
