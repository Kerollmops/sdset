//! Operations for already sorted and deduplicated slices.
//!
//! This library contains to kind of set operations:
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
//! # use sdset::Error;
//! # fn try_main() -> Result<(), Error> {
//! use sdset::duo::OpBuilder;
//! use sdset::SortDedup;
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
//! # use sdset::Error;
//! # fn try_main() -> Result<(), Error> {
//! use sdset::multi::OpBuilder;
//! use sdset::SortDedup;
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

#![cfg_attr(feature = "unstable", feature(test))]

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

#[cfg(all(feature = "unstable", test))]
mod bench {
    mod btree {
        mod difference {
            extern crate test;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
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
            fn two_slices_big2(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }

        mod intersection {
            extern crate test;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
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
            fn two_slices_big2(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.intersection(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.intersection(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }

        mod union {
            extern crate test;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
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
            fn two_slices_big2(bench: &mut Bencher) {
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
            fn two_slices_big3(bench: &mut Bencher) {
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
            fn three_slices_big(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b);
                let c = BTreeSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }
    }

    mod vec {
        mod union {
            extern crate test;
            use self::test::Bencher;

            fn create_vec_set<T: Ord + Clone>(slices: &[&[T]]) -> Vec<T> {
                let alloc = slices.iter().map(|v| v.len()).sum();
                let mut set = Vec::with_capacity(alloc);
                for slice in slices {
                    set.extend_from_slice(slice);
                }
                set.sort_unstable();
                set.dedup();
                set
            }

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b]);
                    test::black_box(|| elements.len());
                });
            }

            #[bench]
            fn two_slices_big2(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b]);
                    test::black_box(|| elements.len());
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b]);
                    test::black_box(|| elements.len());
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b, &c]);
                    test::black_box(|| elements.len());
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b, &c]);
                    test::black_box(|| elements.len());
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                bench.iter(|| {
                    let elements = create_vec_set(&[&a, &b, &c]);
                    test::black_box(|| elements.len());
                });
            }
        }
    }
}
