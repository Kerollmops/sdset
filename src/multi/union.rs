use std::marker;

use crate::SetOperation;
use crate::algorithm::{Algorithm, Exponential};
use crate::set::{Set, vec_sets_into_slices};
use crate::two_minimums::{two_minimums, Minimums::*};

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
pub struct Union<'a, T: 'a, A: Algorithm = Exponential> {
    slices: Vec<&'a [T]>,
    _algo: marker::PhantomData<A>,
}

impl<'a, T, A: Algorithm> Union<'a, T, A> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<&'a Set<T>>) -> Self {
        Self {
            slices: vec_sets_into_slices(slices),
            _algo: marker::PhantomData,
        }
    }
}

impl<'a, T: Ord, A: Algorithm> Union<'a, T, A> {
    #[inline]
    fn extend_vec<U, F, G>(mut self, output: &mut Vec<U>, extend: F, push: G)
    where F: Fn(&mut Vec<U>, &'a [T]),
          G: Fn(&mut Vec<U>, &'a T),
    {
        if let Some(len) = self.slices.iter().map(|s| s.len()).max() {
            output.reserve(len);
        }

        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let off = match A::search(self.slices[i], s) {
                            Ok(off) => off,
                            Err(off) => off,
                        };

                        extend(output, &self.slices[i][..off]);
                        self.slices[i] = &self.slices[i][off..];
                    }
                    push(output, s);
                    for slice in &mut self.slices {
                        if slice.first() == Some(s) {
                            *slice = &slice[1..];
                        }
                    }
                },
                One((i, _)) => {
                    extend(output, self.slices[i]);
                    break;
                },
                Nothing => break,
            }
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Union<'a, T> {
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice, |v, x| v.push(x.clone()));
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Union<'a, T> {
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend, Vec::push);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::SetBuf;

    #[test]
    fn no_slice() {
        let union_: SetBuf<i32> = Union::new(vec![]).into_set_buf();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &Set<i32> = Set::new_unchecked(&[]);

        let union_: SetBuf<i32> = Union::new(vec![a]).into_set_buf();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = Set::new_unchecked(&[1, 2, 3]);

        let union_: SetBuf<i32> = Union::new(vec![a]).into_set_buf();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices_equal() {
        let a = Set::new_unchecked(&[1, 2, 3]);
        let b = Set::new_unchecked(&[1, 2, 3]);

        let union_: SetBuf<i32> = Union::new(vec![a, b]).into_set_buf();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices_little() {
        let a = Set::new_unchecked(&[1]);
        let b = Set::new_unchecked(&[2]);

        let union_: SetBuf<i32> = Union::new(vec![a, b]).into_set_buf();
        assert_eq!(&union_[..], &[1, 2]);
    }

    #[test]
    fn two_slices() {
        let a = Set::new_unchecked(&[1, 2, 3]);
        let b = Set::new_unchecked(&[2, 3, 4]);

        let union_: SetBuf<i32> = Union::new(vec![a, b]).into_set_buf();
        assert_eq!(&union_[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn three_slices() {
        let a = Set::new_unchecked(&[1, 2, 3]);
        let b = Set::new_unchecked(&[2, 3, 4]);
        let c = Set::new_unchecked(&[3, 4, 5]);

        let union_: SetBuf<i32> = Union::new(vec![a, b, c]).into_set_buf();
        assert_eq!(&union_[..], &[1, 2, 3, 4, 5]);
    }

    quickcheck! {
        fn qc_union(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let xss: Vec<_> = xss.into_iter().map(SetBuf::from_dirty).collect();

            let x: SetBuf<i32> = {
                let xss = xss.iter().map(AsRef::as_ref).collect();
                Union::new(xss).into_set_buf()
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

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use super::*;
    use self::test::Bencher;
    use crate::set::SetBuf;

    #[bench]
    fn two_slices_big(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((1..101).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b]).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((51..151).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b]).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((100..200).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b]).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((1..101).collect());
        let c = SetBuf::from_dirty((2..102).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b, &c]).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((34..134).collect());
        let c = SetBuf::from_dirty((66..167).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b, &c]).into_set_buf();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let a = SetBuf::from_dirty((0..100).collect());
        let b = SetBuf::from_dirty((100..200).collect());
        let c = SetBuf::from_dirty((200..300).collect());

        bench.iter(|| {
            let union_: SetBuf<i32> = Union::new(vec![&a, &b, &c]).into_set_buf();
            test::black_box(|| union_);
        });
    }
}
