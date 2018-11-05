use crate::set::Set;
use crate::{exponential_offset_ge, SetOperation};

/// Represent the _difference_ set operation that will be applied to two slices.
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
/// let op = OpBuilder::new(a, b).difference();
///
/// let res: SetBuf<i32> = op.into_set_buf();
/// assert_eq!(&res[..], &[1]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct Difference<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> Difference<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self {
            a: a.as_slice(),
            b: b.as_slice(),
        }
    }
}

impl<'a, T: Ord> Difference<'a, T> {
    #[inline]
    fn extend_vec<U, F>(mut self, output: &mut Vec<U>, extend: F)
    where F: Fn(&mut Vec<U>, &'a [T])
    {
        while let Some(first) = self.a.first() {
            self.b = exponential_offset_ge(self.b, first);
            let minimum = self.b.first();

            match minimum {
                Some(min) if min == first => self.a = exponential_offset_ge(&self.a[1..], min),
                Some(min) => {
                    let off = self.a.iter().take_while(|&x| x < min).count();
                    extend(output, &self.a[..off]);

                    self.a = &self.a[off..];
                },
                None => {
                    extend(output, self.a);
                    break;
                },
            }
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<T> for Difference<'a, T> {
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T> for Difference<'a, T> {
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 4];

        let union_: SetBuf<i32> = Difference { a: a, b: b }.into_set_buf();
        assert_eq!(&union_[..], &[1, 3]);
    }

    #[test]
    fn two_slices_special_case() {
        let a = &[1, 2, 3];
        let b = &[3];

        let union_: SetBuf<i32> = Difference { a: a, b: b }.into_set_buf();
        assert_eq!(&union_[..], &[1, 2]);
    }

    quickcheck! {
        fn qc_difference(a: Vec<i32>, b: Vec<i32>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;

            sort_dedup_vec(&mut a);
            sort_dedup_vec(&mut b);

            let x: SetBuf<i32> = Difference { a: &a, b: &b }.into_set_buf();

            let a = BTreeSet::from_iter(a);
            let b = BTreeSet::from_iter(b);
            let y = a.difference(&b);
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
            let difference_: SetBuf<i32> = Difference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| difference_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let difference_: SetBuf<i32> = Difference { a: &a, b: &b }.into_set_buf();
            test::black_box(|| difference_);
        });
    }
}
