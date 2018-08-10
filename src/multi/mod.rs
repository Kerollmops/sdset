//! Contains the types to make set operations on multiple slices.
//!
//! # Examples
//! ```
//! # use setiter::Error;
//! # fn try_main() -> Result<(), Error> {
//! use setiter::multi::OpBuilder;
//! use setiter::SortDedup;
//!
//! let a = SortDedup::new(&[1, 2, 4])?;
//! let b = SortDedup::new(&[2, 3, 5, 7])?;
//! let c = SortDedup::new(&[4, 6, 7])?;
//!
//! let op = OpBuilder::from_vec(vec![a, b, c]).union();
//!
//! let res = op.into_vec();
//! assert_eq!(&res, &[1, 2, 3, 4, 5, 6, 7]);
//! # Ok(()) }
//! # try_main().unwrap();
//! ```

use std::mem;
use sort_dedup::SortDedup;

mod union;
mod intersection;
mod difference;

pub use self::union::Union;
pub use self::intersection::Intersection;
pub use self::difference::Difference;

/// Type used to acquire any number of slices
/// and make a set operation on these slices.
pub struct OpBuilder<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> OpBuilder<'a, T> {
    /// Construct an empty one.
    pub fn new() -> Self {
        Self { slices: Vec::new() }
    }

    /// Construct an empty one with enough space for `capacity` elements or more.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { slices: Vec::with_capacity(capacity) }
    }

    /// Construct it with the content of the given slice.
    ///
    /// Note that no other allocation than the one of the vec given
    /// in parameter is needed for the construction.
    pub fn from_vec(slices: Vec<SortDedup<'a, T>>) -> Self {
        // the SortDedup type is marked as transparent
        // so it is safe to transmute it to the underlying slice
        // transmuting here is done to avoid doing a useless allocation
        Self::from_vec_unchecked(unsafe { mem::transmute(slices) })
    }

    /// Construct it with the content of the given slice
    /// without checking the slices validity.
    ///
    /// Note that is method is called by [`OpBuilder::from_vec`].
    pub fn from_vec_unchecked(slices: Vec<&'a [T]>) -> Self {
        Self { slices }
    }

    /// Reserve additional space for the underlying vec.
    pub fn reserve(&mut self, additional: usize) {
        self.slices.reserve(additional);
    }

    /// Add a new slice that will be used for the future set operation
    /// and consume and return the type.
    pub fn add(mut self, slice: &'a [T]) -> Self {
        self.push(slice);
        self
    }

    /// Push a new slice that will be used for the future set operation.
    pub fn push(&mut self, slice: &'a [T]) {
        self.slices.push(slice);
    }

    /// Prepare the slices for the _union_ set operation.
    pub fn union(self) -> Union<'a, T> {
        Union::new_unchecked(self.slices)
    }

    /// Prepare the slices for the _intersection_ set operation.
    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new_unchecked(self.slices)
    }

    /// Prepare the slices for the _difference_ set operation.
    pub fn difference(self) -> Difference<'a, T> {
        Difference::new_unchecked(self.slices)
    }
}
