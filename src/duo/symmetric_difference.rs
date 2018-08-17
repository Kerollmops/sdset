use std::cmp::Ordering;
use set::Set;
use ::SetOperation;

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
    fn extend_vec<U, F>(mut self, output: &mut Vec<U>, extend: F)
    where F: Fn(&mut Vec<U>, &'a [T])
    {
        loop {
            match (self.a.first(), self.b.first()) {
                (Some(a), Some(b)) => {
                    match a.cmp(b) {
                        Ordering::Less => {
                            let off = self.a.iter().take_while(|&e| e < b).count();
                            extend(output, &self.a[..off]);
                            self.a = &self.a[off..];
                        },
                        Ordering::Equal => {
                            let off = self.a.iter().zip(self.b.iter()).take_while(|(a, b)| a == b).count();
                            self.a = &self.a[off..];
                            self.b = &self.b[off..];
                        },
                        Ordering::Greater => {
                            let off = self.b.iter().take_while(|&e| e < a).count();
                            extend(output, &self.b[..off]);
                            self.b = &self.b[off..];
                        },
                    }
                },
                (None, Some(_)) => {
                    extend(output, self.b);
                    break;
                },
                (Some(_), None) => {
                    extend(output, self.a);
                    break;
                },
                (None, None) => break,
            }
        }
    }
}

impl<'a, T: Ord + Clone> SetOperation<&'a T, T> for SymmetricDifference<'a, T> {
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice)
    }
}

impl<'a, T: Ord> SetOperation<&'a T, &'a T> for SymmetricDifference<'a, T> {
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use set::{sort_dedup_vec, SetBuf};

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

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use super::*;
    use self::test::Bencher;
    use set::SetBuf;

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
