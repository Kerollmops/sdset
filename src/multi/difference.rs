use std::{cmp, mem};
use sort_dedup::SortDedup;
use ::{extend_iter_len, offset_ge};

pub struct Difference<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Difference<'a, T> {
    pub fn new(slices: Vec<SortDedup<'a, T>>) -> Self {
        Self::new_unchecked(unsafe { mem::transmute(slices) })
    }

    pub fn new_unchecked(slices: Vec<&'a [T]>) -> Self {
        Self { slices }
    }
}

impl<'a, T: Ord + Clone> Difference<'a, T> {
    pub fn extend_vec(mut self, output: &mut Vec<T>) {
        let (base, others) = match self.slices.split_first_mut() {
            Some(split) => split,
            None => return,
        };

        while let Some(first) = base.first() {

            let mut minimum = None;
            for slice in others.iter_mut() {
                *slice = offset_ge(slice, first);
                minimum = match (minimum, slice.first()) {
                    (Some(min), Some(first)) => Some(cmp::min(min, first)),
                    (None, Some(first)) => Some(first),
                    (min, _) => min,
                };
            }

            match minimum {
                Some(min) if min == first => *base = offset_ge(&base[1..], min),
                Some(min) => {
                    let iter = base.iter().take_while(|&x| x < min).cloned();
                    let add = extend_iter_len(iter, output);

                    *base = &base[add..];
                },
                None => {
                    output.extend_from_slice(base);
                    break;
                },
            }
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::new();
        self.extend_vec(&mut vec);
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::{self, Bencher};

    #[test]
    fn no_slice() {
        let union_: Vec<i32> = Difference::new_unchecked(vec![]).into_vec();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let intersection_ = Difference::new_unchecked(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let union_ = Difference::new_unchecked(vec![a]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 4];

        let union_ = Difference::new_unchecked(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 3]);
    }

    #[test]
    fn two_slices_special_case() {
        let a = &[1, 2, 3];
        let b = &[3];

        let union_ = Difference::new_unchecked(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3, 6, 7];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5, 7];

        let union_ = Difference::new_unchecked(vec![a, b, c]).into_vec();
        assert_eq!(&union_[..], &[1, 6]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_ = Difference::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn bench_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_ = Difference::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn bench_two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_ = Difference::new_unchecked(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    fn sort_dedup<T: Ord>(x: &mut Vec<T>) {
        x.sort_unstable();
        x.dedup();
    }

    quickcheck! {
        fn qc_difference(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                sort_dedup(xs);
            }

            let x = {
                let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                Difference::new_unchecked(xss).into_vec()
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
