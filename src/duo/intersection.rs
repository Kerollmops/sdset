use std::cmp;
use ::offset_ge;

pub struct Intersection<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T: 'a> Intersection<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        Intersection { a, b }
    }
}

impl<'a, T: 'a + Ord + Clone> Intersection<'a, T> {
    pub fn extend_vec(mut self, output: &mut Vec<T>) {

        while !self.a.is_empty() && !self.b.is_empty() {
            let a = &self.a[0];
            let b = &self.b[0];

            if a == b {
                output.push(a.clone());
                self.a = &self.a[1..];
                self.b = &self.b[1..];
            }
            else {
                let max = cmp::max(a, b);
                self.a = offset_ge(self.a, max);
                self.b = offset_ge(self.b, max);
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
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let intersection_ = Intersection::new(a, b).into_vec();
        assert_eq!(&intersection_[..], &[2, 3]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let intersection_ = Intersection::new(&a, &b).into_vec();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn bench_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let intersection_ = Intersection::new(&a, &b).into_vec();
            test::black_box(|| intersection_);
        });
    }

    #[bench]
    fn bench_two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let union_ = Intersection::new(&a, &b).into_vec();
            test::black_box(|| union_);
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

    #[bench]
    fn bench_btree_two_slices_big2(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        let base = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = base.intersection(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_btree_two_slices_big3(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        let base = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = base.intersection(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    fn sort_dedup<T: Ord>(x: &mut Vec<T>) {
        x.sort_unstable();
        x.dedup();
    }

    quickcheck! {
        fn qc_intersection(a: Vec<i32>, b: Vec<i32>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;

            sort_dedup(&mut a);
            sort_dedup(&mut b);

            let x = Intersection::new(&a, &b).into_vec();

            let a = BTreeSet::from_iter(a);
            let b = BTreeSet::from_iter(b);
            let y = a.intersection(&b);
            let y: Vec<_> = y.cloned().collect();

            x.as_slice() == y.as_slice()
        }
    }
}
