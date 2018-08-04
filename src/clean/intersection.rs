use std::cmp;
use clean::offset_ge;

use self::Equality::*;

pub struct Intersection<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Intersection<'a, T> {
    pub fn new(slices: Vec<&'a [T]>) -> Self {
        Intersection { slices }
    }
}

enum Equality<'a, T: 'a> {
    NotEqual(&'a T),
    Equal(&'a T),
}

#[inline]
fn test_equality<'a, T: 'a + Ord>(slices: &[&'a [T]]) -> Equality<'a, T> {
    let mut is_equal = true;
    let mut max = &slices[0][0];
    for x in slices {
        let x = &x[0];
        if is_equal { is_equal = max == x }
        max = cmp::max(max, x);
    }
    if is_equal { Equal(max) } else { NotEqual(max) }
}

impl<'a, T: 'a + Ord + Clone> Intersection<'a, T> {
    pub fn extend_vec(mut self, output: &mut Vec<T>) {
        if self.slices.is_empty() { return }
        if self.slices.iter().any(|s| s.is_empty()) { return }

        loop {
            match test_equality(&self.slices) {
                Equal(x) => {
                    output.push(x.clone());
                    for slice in &mut self.slices {
                        *slice = &slice[1..];
                        if slice.is_empty() { return }
                    }
                },
                NotEqual(x) => {
                    for slice in &mut self.slices {
                        *slice = offset_ge(slice, x);
                        if slice.is_empty() { return }
                    }
                }
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
        let intersection_: Vec<i32> = Intersection::new(vec![]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let intersection_ = Intersection::new(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let intersection_ = Intersection::new(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let intersection_ = Intersection::new(vec![a, b]).into_vec();
        assert_eq!(&intersection_[..], &[2, 3]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let intersection_ = Intersection::new(vec![a, b, c]).into_vec();
        assert_eq!(&intersection_[..], &[3]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let intersection_ = Intersection::new(vec![&a, &b]).into_vec();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn bench_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let intersection_ = Intersection::new(vec![&a, &b]).into_vec();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn bench_btree_two_slices_big(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        let a = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = a.intersection(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    fn sort_dedup<T: Ord>(x: &mut Vec<T>) {
        x.sort_unstable();
        x.dedup();
    }

    quickcheck! {
        fn qc_intersection(xss: Vec<Vec<i32>>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            // FIXME temporary hack (can have mutable parameters!)
            let mut xss = xss;

            for xs in &mut xss {
                sort_dedup(xs);
            }

            let x = {
                let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                Intersection::new(xss).into_vec()
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
