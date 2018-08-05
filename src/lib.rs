//! Operations for already deduplicated and sorted slices.

#![feature(test)]
extern crate test;

#[cfg(test)]
#[macro_use] extern crate quickcheck;

pub mod multi;
pub mod duo;
