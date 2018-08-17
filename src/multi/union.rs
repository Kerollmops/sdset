use std::mem;
use set::Set;
use self::Minimums::*;
use ::SetOperation;

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
            slices: unsafe { mem::transmute(slices) },
        }
    }
}

impl<'a, T: Ord> Union<'a, T> {
    #[inline]
    fn extend_vec<U, F, G>(mut self, output: &mut Vec<U>, extend: F, push: G)
    where F: Fn(&mut Vec<U>, &'a [T]),
          G: Fn(&mut Vec<U>, &'a T),
    {
        if let Some(slice) = self.slices.first() {
            output.reserve(slice.len());
        }

        loop {
            match two_minimums(&self.slices) {
                Two((i, f), (_, s)) => {
                    if f != s {
                        let off = self.slices[i].iter().take_while(|&e| e < s).count();
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

impl<'a, T: Ord + Clone> SetOperation<&'a T, T> for Union<'a, T> {
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice, |v, x| v.push(x.clone()));
    }
}

impl<'a, T: Ord> SetOperation<&'a T, &'a T> for Union<'a, T> {
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend, |v, x| v.push(x));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use set::{sort_dedup_vec, SetBuf};

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
