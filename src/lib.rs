//! Operations for already deduplicated and sorted slices.

#![feature(test)]
extern crate test;

#[cfg(test)]
#[macro_use] extern crate quickcheck;

pub mod multi;
pub mod duo;

/// Returns the slice but with its start advanced to an element
/// that is greater or equal to the one given in parameter.
#[inline]
fn offset_ge<'a, 'b, T: 'a + PartialOrd>(slice: &'a [T], elem: &'b T) -> &'a [T] {
    match slice.iter().position(|x| x >= elem) {
        Some(pos) => &slice[pos..],
        None => &[],
    }
}
