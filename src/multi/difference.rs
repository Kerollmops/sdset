use std::cmp;
use crate::set::{Set, vec_sets_into_slices};
use crate::{SetOperation, Collection, exponential_offset_ge, exponential_offset_ge_maybe};

/// Represent the _difference_ set operation that will be applied to the slices.
///
/// Note that the difference is all the elements
/// that are in the first slice but not in all the others.
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
/// let op = OpBuilder::from_vec(vec![a, b, c]).difference();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct Difference<'a, T: 'a> {
    base: &'a [T],
    others: Vec<&'a [T]>,
}

impl<'a, T> Difference<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<&'a Set<T>>) -> Self {
        let mut it = slices.iter();
        Self {
            base: match it.next() {
                Some(&x) => {
                    x
                },
                None => {
                    &[]
                }
            },
            others: if slices.len() < 2 {
                vec![]
            } else {
                let mut o = Vec::<&[T]>::with_capacity(slices.len()-1);
                Extend::extend(&mut o, it.filter(|&x| x.len() > 0).map(|&x| x.as_slice()));
                o
            },
        }
    }
}

impl<'a, T: Ord> Difference<'a, T> {
    #[inline]
    fn extend_collection<C, U, F>(mut self, output: &mut C, extend: F) -> Result<(), C::Error>
    where C: Collection<U>,
          F: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
    {
        while let Some(first) = self.base.first() {
            let mut minimum:Option<&T> = None;
            let mut i: usize = 0;
            while i < self.others.len() {
                self.others[i] = exponential_offset_ge_maybe(self.others[i], first);
                if let Some(x) = self.others[i].first() {
                    if let Some(min) = minimum {
                        if x < min {
                            minimum = Some(x);
                        }
                    } else {
                        minimum = Some(x);
                    }
                    i += 1;
                } else {
                    self.others.swap_remove(i);
                }
            }
            
            if let Some(min) = minimum {
                if min == first {
                    self.base = &self.base[1..];
                } else { // minimum > first
                    let off = self.base.iter().take_while(|&x| x < min).count();
                    extend(output, &self.base[..off])?;
    
                    self.base = &self.base[off..];
                }
            } else {
                extend(output, self.base)?;
                break;
            }
        }
        Ok(())
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Difference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Difference<'a, T> {
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    #[test]
    fn no_slice() {
        let difference_: SetBuf<i32> = Difference::new(vec![]).into_set_buf();
        assert_eq!(&difference_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &Set<i32> = Set::<i32>::new_unchecked(&[]);

        let difference_: SetBuf<i32> = Difference::new(vec![a]).into_set_buf();
        assert_eq!(&difference_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = Set::new_unchecked(&[1, 2, 3]);

        let difference_: SetBuf<i32> = Difference::new(vec![a]).into_set_buf();
        assert_eq!(&difference_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = Set::new_unchecked(&[1, 2, 3]);
        let b = Set::new_unchecked(&[2, 4]);

        let difference_: SetBuf<i32> = Difference::new(vec![a,b]).into_set_buf();
        assert_eq!(&difference_[..], &[1, 3]);
    }

    #[test]
    fn two_slices_special_case() {
        let a = Set::new_unchecked(&[1, 2, 3]);
        let b = Set::new_unchecked(&[3]);

        let difference_: SetBuf<i32> = Difference::new(vec![a, b]).into_set_buf();
        assert_eq!(&difference_[..], &[1, 2]);
    }

    #[test]
    fn three_slices() {
        let a = Set::new_unchecked(&[1, 2, 3, 6, 7]);
        let b = Set::new_unchecked(&[2, 3, 4]);
        let c = Set::new_unchecked(&[3, 4, 5, 7]);

        let difference_: SetBuf<i32> = Difference::new(vec![a, b, c]).into_set_buf();
        assert_eq!(&difference_[..], &[1, 6]);
    }
    
    quickcheck! {
        fn qc_difference(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                sort_dedup_vec(xs);
            }

            let x: SetBuf<i32> = {
                let xss: Vec<&Set<i32>> = xss.iter().map(|xs| Set::new_unchecked(&xs[..])).collect();
                Difference::new(xss).into_set_buf()
            };

            let mut xss = xss.into_iter();
            let mut y = match xss.next() {
                Some(xs) => BTreeSet::from_iter(xs),
                None => BTreeSet::new(),
            };

            for v in xss {
                let x = BTreeSet::from_iter(v.iter().cloned());
                y = y.difference(&x).cloned().collect();
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
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (1..101).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (51..151).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (100..200).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (1..101).collect();
        let cv: Vec<_> = (2..102).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());
        let c = Set::new_unchecked(cv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b,c]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (34..134).collect();
        let cv: Vec<_> = (66..167).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());
        let c = Set::new_unchecked(cv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b,c]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();
        let bv: Vec<_> = (100..200).collect();
        let cv: Vec<_> = (200..300).collect();
        let a = Set::new_unchecked(av.as_slice());
        let b = Set::new_unchecked(bv.as_slice());
        let c = Set::new_unchecked(cv.as_slice());

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference::new(vec![a,b,c]).into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn hundred_small_slices(bench: &mut Bencher) {
        let av: Vec<_> = (0..100).collect();

        bench.iter(|| {
            let mut sets: Vec<&Set<i32>> = Vec::with_capacity(101);
            let a = Set::new_unchecked(av.as_slice());
            sets.push(a);
            for i in 0..100 {
                let b = Set::new_unchecked(&av[i..i+1]);
                sets.push(b);
            }
            let difference_: SetBuf<i32> = Difference::new(sets).into_set_buf();
            test::black_box(|| difference_);
        });
    }
}
