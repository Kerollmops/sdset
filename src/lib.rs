//! Operations for already sorted and deduplicated slices.
//!
//! This library contains two modules containing types to produce set operations:
//!   - The [`duo`] module is for types limited to be used with two slices not more not less.
//!   - The [`multi`] module types can be used to do set operations on multiple slices from zero up to an infinite number.
//!
//! The [`duo`] operations are much more performant than [`multi`]
//! so prefer using [`duo`] when you know that you will need set operations for two slices.
//!
//! # Examples
//!
//! Using a [`duo`] _union_ set operation on two slices.
//!
//! ```
//! # use sdset::Error;
//! # fn try_main() -> Result<(), Error> {
//! use sdset::duo::OpBuilder;
//! use sdset::{SetOperation, Set, SetBuf};
//!
//! let a = Set::new(&[1, 2, 4, 6, 7])?;
//! let b = Set::new(&[2, 3, 4, 5, 6, 7])?;
//!
//! let op = OpBuilder::new(a, b).union();
//!
//! let res: SetBuf<i32> = op.into_set_buf();
//! assert_eq!(&res[..], &[1, 2, 3, 4, 5, 6, 7]);
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
//! use sdset::{SetOperation, Set, SetBuf};
//!
//! let a = Set::new(&[1, 2, 4])?;
//! let b = Set::new(&[2, 3, 4, 5, 7])?;
//! let c = Set::new(&[2, 4, 6, 7])?;
//!
//! let op = OpBuilder::from_vec(vec![a, b, c]).intersection();
//!
//! let res: SetBuf<i32> = op.into_set_buf();
//! assert_eq!(&res[..], &[2, 4]);
//! # Ok(()) }
//! # try_main().unwrap();
//! ```

#![warn(missing_docs)]

#![cfg_attr(feature = "unstable", feature(test))]

#[cfg(test)]
#[macro_use] extern crate quickcheck;

pub mod duo;
pub mod multi;
pub mod set;
mod two_minimums;

use std::cmp::{self, Ordering};
pub use set::{Set, SetBuf, Error};

/// Exponential searches this sorted slice for a given element.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use sdset::exponential_search;
///
/// let s = &[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
///
/// assert_eq!(exponential_search(s, &13),  Ok(9));
/// assert_eq!(exponential_search(s, &4),   Err(7));
/// assert_eq!(exponential_search(s, &100), Err(13));
/// let r = exponential_search(s, &1);
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search<T>(slice: &[T], elem: &T) -> Result<usize, usize>
where T: Ord
{
    exponential_search_by(slice, |x| x.cmp(elem))
}

/// Binary searches this sorted slice with a comparator function.
///
/// The comparator function should implement an order consistent with the sort order of
/// the underlying slice, returning an order code that indicates whether its argument
/// is `Less`, `Equal` or `Greater` the desired target.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use sdset::exponential_search_by;
///
/// let s = &[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
///
/// let seek = 13;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Ok(9));
/// let seek = 4;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Err(7));
/// let seek = 100;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Err(13));
/// let seek = 1;
/// let r = exponential_search_by(s, |probe| probe.cmp(&seek));
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search_by<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
where F: FnMut(&T) -> Ordering,
{
    let mut index = 1;
    while index < slice.len() && f(&slice[index]) == Ordering::Less {
        index *= 2;
    }

    let half_bound = index / 2;
    let bound = cmp::min(index + 1, slice.len());

    match slice[half_bound..bound].binary_search_by(f) {
        Ok(pos) => Ok(half_bound + pos),
        Err(pos) => Err(half_bound + pos),
    }
}

/// Binary searches this sorted slice with a key extraction function.
///
/// Assumes that the slice is sorted by the key.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use sdset::exponential_search_by_key;
///
/// let s = &[(0, 0), (2, 1), (4, 1), (5, 1), (3, 1),
///           (1, 2), (2, 3), (4, 5), (5, 8), (3, 13),
///           (1, 21), (2, 34), (4, 55)];
///
/// assert_eq!(exponential_search_by_key(s, &13, |&(a,b)| b),  Ok(9));
/// assert_eq!(exponential_search_by_key(s, &4, |&(a,b)| b),   Err(7));
/// assert_eq!(exponential_search_by_key(s, &100, |&(a,b)| b), Err(13));
/// let r = exponential_search_by_key(s, &1, |&(a,b)| b);
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search_by_key<T, B, F>(slice: &[T], b: &B, mut f: F) -> Result<usize, usize>
where F: FnMut(&T) -> B,
      B: Ord
{
    exponential_search_by(slice, |k| f(k).cmp(b))
}

#[inline]
fn exponential_offset_ge<'a, T>(slice: &'a [T], elem: &T) -> &'a [T]
where T: Ord,
{
    exponential_offset_ge_by(slice, |x| x.cmp(elem))
}

#[inline]
fn exponential_offset_ge_by<T, F>(slice: &[T], f: F) -> &[T]
where F: FnMut(&T) -> Ordering,
{
    match exponential_search_by(slice, f) {
        Ok(pos) => &slice[pos..],
        Err(pos) => &slice[pos..],
    }
}

#[inline]
fn exponential_offset_ge_by_key<'a, T, B, F>(slice: &'a [T], b: &B, mut f: F) -> &'a [T]
where F: FnMut(&T) -> B,
      B: Ord,
{
    exponential_offset_ge_by(slice, |x| f(x).cmp(b))
}

/// Represent a type that can produce a set operation on multiple [`Set`]s.
pub trait SetOperation<T>: Sized {
    /// Extend a [`Vec`] with the values of the [`Set`]s using this set operation.
    fn extend_vec(self, output: &mut Vec<T>);

    /// Create a [`SetBuf`] using the [`SetOperation::extend_vec`] method.
    fn into_set_buf(self) -> SetBuf<T> {
        let mut vec = Vec::new();
        self.extend_vec(&mut vec);
        SetBuf::new_unchecked(vec)
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    mod _btree {
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

        mod symmetric_difference {
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
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
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
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
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
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
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
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
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
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
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
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }
    }

    mod _fnv {
        mod difference {
            extern crate test;
            extern crate fnv;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a - &b;
                    let set: Vec<_> = ab.difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }

        mod intersection {
            extern crate test;
            extern crate fnv;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.intersection(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.intersection(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.intersection(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a & &b;
                    let set: Vec<_> = ab.intersection(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }

        mod union {
            extern crate test;
            extern crate fnv;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.union(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.union(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.union(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a | &b;
                    let set: Vec<_> = ab.union(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }

        mod symmetric_difference {
            extern crate test;
            extern crate fnv;
            use self::test::Bencher;

            #[bench]
            fn two_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (51..151).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn two_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);

                bench.iter(|| {
                    let set: Vec<_> = a.symmetric_difference(&b).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (1..101).collect();
                let c: Vec<_> = (2..102).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big2(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (34..134).collect();
                let c: Vec<_> = (67..167).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }

            #[bench]
            fn three_slices_big3(bench: &mut Bencher) {
                use self::fnv::FnvHashSet;
                use std::iter::FromIterator;

                let a: Vec<_> = (0..100).collect();
                let b: Vec<_> = (100..200).collect();
                let c: Vec<_> = (200..300).collect();

                let a: FnvHashSet<i32> = FnvHashSet::from_iter(a);
                let b = FnvHashSet::from_iter(b);
                let c = FnvHashSet::from_iter(c);

                bench.iter(|| {
                    let ab = &a ^ &b;
                    let set: Vec<_> = ab.symmetric_difference(&c).cloned().collect();
                    test::black_box(|| set);
                });
            }
        }
    }

    mod _vec {
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
