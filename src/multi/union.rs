use std::mem;
use sort_dedup::SortDedup;
use self::Minimums::*;

/// Represent the _union_ set operation that will be applied to the slices.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::multi::OpBuilder;
/// use sdset::SortDedup;
///
/// let a = SortDedup::new(&[1, 2, 4])?;
/// let b = SortDedup::new(&[2, 3, 5, 7])?;
/// let c = SortDedup::new(&[4, 6, 7])?;
///
/// let op = OpBuilder::from_vec(vec![a, b, c]).union();
///
/// let res = op.into_vec();
/// assert_eq!(&res, &[1, 2, 3, 4, 5, 6, 7]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct Union<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Union<'a, T> {
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(slices: Vec<SortDedup<'a, T>>) -> Self {
        Self::new_unchecked(unsafe { mem::transmute(slices) })
    }

    /// Construct one with unchecked slices.
    pub fn new_unchecked(slices: Vec<&'a [T]>) -> Self {
        Self { slices }
    }
}

enum Minimums<T> {
    Nothing,
    One(T),
    Two(T, T),
}

/// Returns the first values of two slices along with the indexes
/// which are the minimums (could be equal).
#[inline]
fn two_minimums<'a, T: 'a + Ord>(slices: &[&'a [T]]) -> Minimums<(usize, &'a T)> {

    let mut minimums: Minimums<(_, &T)> = Nothing;

    for (index, slice) in slices.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        let current = (index, &slice[0]);
        let (_, min) = current;

        minimums = match minimums {
            One(f) | Two(f, _) if min <  f.1 => Two(current, f),
            One(f)             if min >= f.1 => Two(f, current),
            Two(f, s)          if min <  s.1 => Two(f, current),
            Nothing                          => One(current),
            other                            => other,
        };
    }

    minimums
}

impl<'a, T: Ord + Clone> Union<'a, T> {
    /// Extend a [`Vec`] with the cloned values of the slices using the set operation.
    pub fn extend_vec(mut self, output: &mut Vec<T>) {
        if let Some(slice) = self.slices.first() {
            output.reserve(slice.len());
        }

        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let off = self.slices[i].iter().take_while(|&e| e < s).count();
                        output.extend_from_slice(&self.slices[i][..off]);
                        self.slices[i] = &self.slices[i][off..];
                    }
                    output.push(s.clone());
                    for slice in &mut self.slices {
                        if slice.first() == Some(s) {
                            *slice = &slice[1..];
                        }
                    }
                },
                One((i, _)) => {
                    output.extend_from_slice(self.slices[i]);
                    break;
                },
                Nothing => break,
            }
        }
    }

    /// Populate a [`Vec`] with the cloned values of the slices using the set operation.
    pub fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::new();
        self.extend_vec(&mut vec);
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_slice() {
        let union_: Vec<i32> = Union::new_unchecked(vec![]).into_vec();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let union_ = Union::new_unchecked(vec![a]).into_vec();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let union_ = Union::new_unchecked(vec![a]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices_equal() {
        let a = &[1, 2, 3];
        let b = &[1, 2, 3];

        let union_ = Union::new_unchecked(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices_little() {
        let a = &[1];
        let b = &[2];

        let union_ = Union::new_unchecked(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union_ = Union::new_unchecked(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let union_ = Union::new_unchecked(vec![a, b, c]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3, 4, 5]);
    }

    quickcheck! {
        fn qc_union(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                ::sort_dedup_vec(xs);
            }

            let x = {
                let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                Union::new_unchecked(xss).into_vec()
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

    #[bench]
    fn two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_ = Union::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }
}
