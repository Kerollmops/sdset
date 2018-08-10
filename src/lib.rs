//! Operations for already deduplicated and sorted slices.
//!
//! This library contains to types of set operations:
//!   - The [`duo`] is limited to be used with two slices not more not less.
//! The operations are much more performant than the [`multi`].
//!   - The [`multi`] can be used to do set operations on multiple slices from zero up to an infinite number.
//!
//! So prefer using the [`duo`] when you know that you will need set operations for two slices.
//!
//! # Examples
//!
//! Using a [`duo`] _union_ set operation on two slices.
//!
//! ```
//! # use setiter::Error;
//! # fn try_main() -> Result<(), Error> {
//! use setiter::duo::OpBuilder;
//! use setiter::SortDedup;
//!
//! let a = SortDedup::new(&[1, 2, 4, 6, 7])?;
//! let b = SortDedup::new(&[2, 3, 4, 5, 6, 7])?;
//!
//! let op = OpBuilder::new(a, b).union();
//!
//! let res = op.into_vec();
//! assert_eq!(&res, &[1, 2, 3, 4, 5, 6, 7]);
//! # Ok(()) }
//! # try_main().unwrap();
//! ```
//!
//! Using a [`multi`] _intersection_ set operation on three slices.
//!
//! ```
//! # use setiter::Error;
//! # fn try_main() -> Result<(), Error> {
//! use setiter::multi::OpBuilder;
//! use setiter::SortDedup;
//!
//! let a = SortDedup::new(&[1, 2, 4])?;
//! let b = SortDedup::new(&[2, 3, 4, 5, 7])?;
//! let c = SortDedup::new(&[2, 4, 6, 7])?;
//!
//! let op = OpBuilder::from_vec(vec![a, b, c]).intersection();
//!
//! let res = op.into_vec();
//! assert_eq!(&res, &[2, 4]);
//! # Ok(()) }
//! # try_main().unwrap();
//! ```

#![warn(missing_docs)]

#![feature(test)]
extern crate test;

#[cfg(test)]
#[macro_use] extern crate quickcheck;

mod sort_dedup;
pub mod multi;
pub mod duo;

pub use sort_dedup::{
    SortDedup, Error,
    sort_dedup_vec,
    is_sort_dedup,
};

/// Returns the slice but with its start advanced to an element
/// that is greater or equal to the one given in parameter.
#[inline]
fn offset_ge<'a, 'b, T: 'a + PartialOrd>(slice: &'a [T], elem: &'b T) -> &'a [T] {
    match slice.iter().position(|x| x >= elem) {
        Some(pos) => &slice[pos..],
        None => &[],
    }
}

fn extend_iter_len<I, T>(iter: I, vec: &mut Vec<T>) -> usize
where I: IntoIterator<Item=T>
{
    let len = vec.len();
    vec.extend(iter);
    vec.len() - len
}

#[cfg(test)]
mod tests {
    use test::{self, Bencher};

    #[bench]
    fn bench_difference_btree_two_slices_big(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        let a = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = a.difference(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_difference_btree_two_slices_big2(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        let base = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = base.difference(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_difference_btree_two_slices_big3(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        let base = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = base.difference(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_intersection_btree_two_slices_big(bench: &mut Bencher) {
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
    fn bench_intersection_btree_two_slices_big2(bench: &mut Bencher) {
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
    fn bench_intersection_btree_two_slices_big3(bench: &mut Bencher) {
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

    #[bench]
    fn bench_union_btree_two_slices_big(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        let a = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = a.union(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_union_btree_two_slices_big2(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        let a = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = a.union(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_union_btree_two_slices_big3(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        let a = BTreeSet::from_iter(a);
        let b = BTreeSet::from_iter(b);

        bench.iter(|| {
            let set: Vec<_> = a.union(&b).cloned().collect();
            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_union_sort_dedup_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let mut elements: Vec<_> = vec![&a, &b].into_iter().flatten().cloned().collect();
            elements.sort_unstable();
            elements.dedup();

            test::black_box(|| elements);
        });
    }

    #[bench]
    fn bench_union_sort_dedup_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let mut elements: Vec<_> = vec![&a, &b].into_iter().flatten().cloned().collect();
            elements.sort_unstable();
            elements.dedup();

            test::black_box(|| elements);
        });
    }

    #[bench]
    fn bench_union_sort_dedup_two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let mut elements: Vec<_> = vec![&a, &b].into_iter().flatten().cloned().collect();
            elements.sort_unstable();
            elements.dedup();

            test::black_box(|| elements);
        });
    }
}
