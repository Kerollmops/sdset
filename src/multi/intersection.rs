use std::cmp;
use crate::set::{Set, vec_sets_into_slices};
use crate::{SetOperation, Collection, exponential_offset_ge};

use self::Equality::*;

/// Represent the _intersection_ set operation that will be applied to the slices.
///
/// Note that the intersection is all the elements that are in all the slices at the same time.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::multi::OpBuilder;
/// use sdset::{SetOperation, Set, SetBuf};
///
/// let a = Set::new(&[1, 2, 4])?;
/// let b = Set::new(&[2, 3, 4, 5, 7])?;
/// let c = Set::new(&[2, 4, 6, 7])?;
///
/// let op = OpBuilder::from_vec(vec![a, b, c]).intersection();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[2, 4]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct Intersection<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Intersection<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<&'a Set<T>>) -> Self {
        Self {
            slices: vec_sets_into_slices(slices),
        }
    }
}

enum Equality<'a, T: 'a> {
    NotEqual(&'a T),
    Equal(&'a T),
}

#[inline]
fn test_equality<'a, T: Ord>(slices: &[&'a [T]]) -> Equality<'a, T> {
    let mut is_equal = true;
    let mut max = &slices[0][0];
    for s in slices {
        let x = &s[0];
        if x != max {
            is_equal = false;
            if x > max {
                max = x;
            }
        }
    }
    if is_equal { Equal(max) } else { NotEqual(max) }
}

impl<'a, T: Ord> Intersection<'a, T> {
    #[inline]
    fn extend_collection<C, U, F>(mut self, output: &mut C, push: F) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a T) -> Result<(), C::Error>,
    {
        if self.slices.is_empty() { return Ok(()) }
        if self.slices.iter().any(|s| s.is_empty()) { return Ok(()) }

        loop {
            match test_equality(&self.slices) {
                Equal(x) => {
                    push(output, x)?;
                    for slice in &mut self.slices {
                        *slice = &slice[1..];
                        if slice.is_empty() { return Ok(()) }
                    }
                },
                NotEqual(max) => {
                    for slice in &mut self.slices {
                        if &slice[0] != max {
                            *slice = exponential_offset_ge(slice, max);
                            if slice.is_empty() { return Ok(()) }
                        }
                    }
                }
            }
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Intersection<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, |v, x| v.push(x.clone()))
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Intersection<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::push)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    #[test]
    fn no_slice() {
        let intersection_: SetBuf<i32> = Intersection { slices: vec![] }.into_set_buf();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let intersection_: SetBuf<i32> = Intersection { slices: vec![a] }.into_set_buf();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let intersection_: SetBuf<i32> = Intersection { slices: vec![a] }.into_set_buf();
        assert_eq!(&intersection_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let intersection_: SetBuf<i32> = Intersection { slices: vec![a, b] }.into_set_buf();
        assert_eq!(&intersection_[..], &[2, 3]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let intersection_: SetBuf<i32> = Intersection { slices: vec![a, b, c] }.into_set_buf();
        assert_eq!(&intersection_[..], &[3]);
    }

    quickcheck! {
        fn qc_intersection(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                sort_dedup_vec(xs);
            }

            let x: SetBuf<i32> = {
                let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                Intersection { slices: xss }.into_set_buf()
            };

            let mut xss = xss.into_iter();
            let mut y = match xss.next() {
                Some(xs) => BTreeSet::from_iter(xs),
                None => BTreeSet::new(),
            };

            for v in xss {
                let x = BTreeSet::from_iter(v.iter().cloned());
                y = y.intersection(&x).cloned().collect();
            }
            let y: Vec<_> = y.into_iter().collect();

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
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();
        let c: Vec<_> = (2..102).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (34..134).collect();
        let c: Vec<_> = (66..167).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();
        let c: Vec<_> = (200..300).collect();

        bench.iter(|| {
            let intersection_: SetBuf<i32> = Intersection { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| intersection_);
        });
    }
}
