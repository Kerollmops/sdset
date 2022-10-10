use crate::set::{Set, vec_sets_into_slices};
use crate::two_minimums::{two_minimums, Minimums::*};
use crate::{SetOperation, Collection};

/// Represent the _union_ set operation that will be applied to the slices.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::multi::OpBuilder;
/// use sdset::{SetOperation, Set, SetBuf};
///
/// let a = Set::new(&[1, 2, 4])?;
/// let b = Set::new(&[2, 3, 5, 7])?;
/// let c = Set::new(&[4, 6, 7])?;
///
/// let op = OpBuilder::from_vec(vec![a, b, c]).union();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1, 2, 3, 4, 5, 6, 7]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct Union<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Union<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<&'a Set<T>>) -> Self {
        Self {
            slices: vec_sets_into_slices(slices),
        }
    }
}

impl<'a, T: Ord> Union<'a, T> {
    #[inline]
    fn extend_collection<C, U, F, G>(mut self, output: &mut C, extend: F, push: G) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
          G: Fn(&mut C, &'a T) -> Result<(), C::Error>,
    {
        if let Some(slice) = self.slices.first() {
            output.reserve(slice.len())?;
        }

        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let off = self.slices[i].iter().take_while(|&e| e < s).count();
                        extend(output, &self.slices[i][..off])?;
                        self.slices[i] = &self.slices[i][off..];
                    }
                    push(output, s)?;
                    for slice in &mut self.slices {
                        if slice.first() == Some(s) {
                            *slice = &slice[1..];
                        }
                    }
                },
                One((i, _)) => {
                    extend(output, self.slices[i])?;
                    break;
                },
                Nothing => break,
            }
        }
        Ok(())
    }

    fn iter(&self) -> UnionIter<'a, T>
    {
        UnionIter {
            slices: self.slices.clone(),
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice, |v, x| v.push(x.clone()))
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Union<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend, Collection::push)
    }
}

impl<'a, T: Ord> IntoIterator for Union<'a, T> {
    type Item = &'a T;
    type IntoIter = UnionIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        UnionIter {
            slices: self.slices,
        }
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
    slices: Vec<&'a [T]>,
}

impl<'a, T: Ord> Iterator for UnionIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let result = &self.slices[i][0];
                        self.slices[i] = &self.slices[i][1..];
                        return Some(result);
                    } else {
                        for slice in &mut self.slices {
                            if slice.first() == Some(s) {
                                *slice = &slice[1..];
                            }
                        }
                        return Some(s);
                    }
                    
                },
                One((i, _)) => {
                    let result = &self.slices[i][0];
                    self.slices[i] = &self.slices[i][1..];
                    return Some(result);
                },
                Nothing => { return None; },
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
        fn no_slice() {
            let union_: SetBuf<i32> = Union { slices: vec![] }.into_set_buf();
            assert_eq!(&union_[..], &[]);
        }

        #[test]
        fn one_empty_slice() {
            let a: &[i32] = &[];

            let union_: SetBuf<i32> = Union { slices: vec![a] }.into_set_buf();
            assert_eq!(&union_[..], &[]);
        }

        #[test]
        fn one_slice() {
            let a = &[1, 2, 3];

            let union_: SetBuf<i32> = Union { slices: vec![a] }.into_set_buf();
            assert_eq!(&union_[..], &[1, 2, 3]);
        }

        #[test]
        fn two_slices_equal() {
            let a = &[1, 2, 3];
            let b = &[1, 2, 3];

            let union_: SetBuf<i32> = Union { slices: vec![a, b] }.into_set_buf();
            assert_eq!(&union_[..], &[1, 2, 3]);
        }

        #[test]
        fn two_slices_little() {
            let a = &[1];
            let b = &[2];

            let union_: SetBuf<i32> = Union { slices: vec![a, b] }.into_set_buf();
            assert_eq!(&union_[..], &[1, 2]);
        }

        #[test]
        fn two_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let union_: SetBuf<i32> = Union { slices: vec![a, b] }.into_set_buf();
            assert_eq!(&union_[..], &[1, 2, 3, 4]);
        }

        #[test]
        fn three_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];
            let c = &[3, 4, 5];

            let union_: SetBuf<i32> = Union { slices: vec![a, b, c] }.into_set_buf();
            assert_eq!(&union_[..], &[1, 2, 3, 4, 5]);
        }

        quickcheck! {
            fn qc_union(xss: Vec<Vec<i32>>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                // FIXME temporary hack (can have mutable parameters!)
                let mut xss = xss;

                for xs in &mut xss {
                    sort_dedup_vec(xs);
                }

                let x: SetBuf<i32> = {
                    let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                    Union { slices: xss }.into_set_buf()
                };

                let mut y = BTreeSet::new();
                for v in xss {
                    let x = BTreeSet::from_iter(v.iter().cloned());
                    y = y.union(&x).cloned().collect();
                }
                let y: Vec<_> = y.into_iter().collect();

                x.as_slice() == y.as_slice()
            }
        }
    }
    
    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        #[test]
        fn no_slice() {
            let union = Union { slices: vec![] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[]);
        }

        #[test]
        fn one_empty_slice() {
            let a: &[i32] = &[];

            let union = Union { slices: vec![a] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[]);
        }

        #[test]
        fn one_slice() {
            let a = &[1, 2, 3];

            let union = Union { slices: vec![a] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[1, 2, 3]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[1, 2, 3]);
        }

        #[test]
        fn two_slices_equal() {
            let a = &[1, 2, 3];
            let b = &[1, 2, 3];
            
            let union = Union { slices: vec![a, b] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[1, 2, 3]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[1, 2, 3]);
        }

        #[test]
        fn two_slices_little() {
            let a = &[1];
            let b = &[2];

            let union = Union { slices: vec![a, b] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[1, 2]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[1, 2]);
        }

        #[test]
        fn two_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let union = Union { slices: vec![a, b] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[1, 2, 3, 4]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[1, 2, 3, 4]);
        }

        #[test]
        fn three_slices() {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];
            let c = &[3, 4, 5];

            let union = Union { slices: vec![a, b, c] };
            let union_ref: Vec<i32> = union.iter().cloned().collect();
            assert_eq!(&union_ref[..], &[1, 2, 3, 4, 5]);
            let union_own: Vec<i32> = union.into_iter().cloned().collect();
            assert_eq!(&union_own[..], &[1, 2, 3, 4, 5]);
        }

        quickcheck! {
            fn qc_union(xss: Vec<Vec<i32>>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                // FIXME temporary hack (can have mutable parameters!)
                let mut xss = xss;

                for xs in &mut xss {
                    sort_dedup_vec(xs);
                }

                let x: Vec<i32> = {
                    let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                    Union { slices: xss }.into_iter().cloned().collect()
                };

                let mut y = BTreeSet::new();
                for v in xss {
                    let x = BTreeSet::from_iter(v.iter().cloned());
                    y = y.union(&x).cloned().collect();
                }
                let y: Vec<_> = y.into_iter().collect();

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
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();
        let c: Vec<_> = (2..102).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (34..134).collect();
        let c: Vec<_> = (66..167).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();
        let c: Vec<_> = (200..300).collect();

        bench.iter(|| {
            let union_: SetBuf<i32> = Union { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| union_);
        });
    }
}
