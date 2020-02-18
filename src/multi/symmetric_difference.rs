use crate::set::{Set, vec_sets_into_slices};
use crate::two_minimums::{two_minimums, Minimums::*};
use crate::{SetOperation, Collection};

/// Represent the _symmetric difference_ set operation that will be applied to the slices.
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
/// let op = OpBuilder::from_vec(vec![a, b, c]).symmetric_difference();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1, 3, 5, 6]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct SymmetricDifference<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> SymmetricDifference<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<&'a Set<T>>) -> Self {
        Self {
            slices: vec_sets_into_slices(slices),
        }
    }
}

impl<'a, T: Ord> SymmetricDifference<'a, T> {
    #[inline]
    fn extend_collection<C, U, F, G>(mut self, output: &mut C, extend: F, push: G)
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]),
          G: Fn(&mut C, &'a T),
    {
        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let off = self.slices[i].iter().take_while(|&e| e < s).count();
                        extend(output, &self.slices[i][..off]);
                        self.slices[i] = &self.slices[i][off..];
                    }
                    else {
                        let mut count = 0;
                        for slice in self.slices.iter_mut() {
                            if slice.first() == Some(f) {
                                count += 1;
                                *slice = &slice[1..];
                            }
                        }
                        // if count is odd
                        if count % 2 != 0 {
                            push(output, f);
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

impl<'a, T: Ord + Clone> SetOperation<T> for SymmetricDifference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) where C: Collection<T> {
        self.extend_collection(output, Collection::extend_from_slice, |v, x| v.push(x.clone()));
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for SymmetricDifference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) where C: Collection<&'a T> {
        self.extend_collection(output, Collection::extend, Collection::push);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    quickcheck! {
        fn qc_symmetric_difference(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                sort_dedup_vec(xs);
            }

            let x: SetBuf<i32> = {
                let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                SymmetricDifference { slices: xss }.into_set_buf()
            };

            let mut y = BTreeSet::new();
            for v in xss {
                let x = BTreeSet::from_iter(v.iter().cloned());
                y = y.symmetric_difference(&x).cloned().collect();
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
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();
        let c: Vec<_> = (2..102).collect();

        bench.iter(|| {
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (34..134).collect();
        let c: Vec<_> = (66..167).collect();

        bench.iter(|| {
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();
        let c: Vec<_> = (200..300).collect();

        bench.iter(|| {
            let symdiff_: SetBuf<i32> = SymmetricDifference { slices: vec![&a, &b, &c] }.into_set_buf();
            test::black_box(|| symdiff_);
        });
    }
}
