//! Contains the types to make set operations on two slices and only two.
//!
//! # Examples
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

use sort_dedup::SortDedup;

mod union;
mod difference;
mod intersection;

pub use self::union::Union;
pub use self::difference::Difference;
pub use self::intersection::Intersection;

/// Type used to make a set operation on two slices only.
#[derive(Copy, Clone)]
pub struct OpBuilder<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> OpBuilder<'a, T> {
    /// Construct a type with two slices.
    pub fn new(a: SortDedup<'a, T>, b: SortDedup<'a, T>) -> Self {
        Self::new_unchecked(a.into_slice(), b.into_slice())
    }

    /// Construct a type with two slices that are not checked to be sorted and deduplicated.
    pub fn new_unchecked(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b }
    }

    /// Prepare the two slices for the _union_ set operation.
    pub fn union(self) -> Union<'a, T> {
        Union::new_unchecked(self.a, self.b)
    }

    /// Prepare the two slices for the _intersection_ set operation.
    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new_unchecked(self.a, self.b)
    }

    /// Prepare the two slices for the _difference_ set operation.
    pub fn difference(self) -> Difference<'a, T> {
        Difference::new_unchecked(self.a, self.b)
    }
}
