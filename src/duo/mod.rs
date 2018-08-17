//! Contains the types to make set operations on two slices and only two.
//!
//! # Examples
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

use set::Set;

mod union;
mod difference;
mod intersection;
mod symmetric_difference;

pub use self::union::Union;
pub use self::difference::Difference;
pub use self::intersection::Intersection;
pub use self::symmetric_difference::SymmetricDifference;

/// Type used to make a set operation on two slices only.
#[derive(Copy, Clone)]
pub struct OpBuilder<'a, T: 'a> {
    a: &'a Set<T>,
    b: &'a Set<T>,
}

impl<'a, T> OpBuilder<'a, T> {
    /// Construct a type with two slices.
    pub fn new(a: &'a Set<T>, b: &'a Set<T>) -> Self {
        Self { a, b }
    }

    /// Prepare the two slices for the _union_ set operation.
    pub fn union(self) -> Union<'a, T> {
        Union::new(self.a, self.b)
    }

    /// Prepare the two slices for the _intersection_ set operation.
    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new(self.a, self.b)
    }

    /// Prepare the two slices for the _difference_ set operation.
    pub fn difference(self) -> Difference<'a, T> {
        Difference::new(self.a, self.b)
    }

    /// Prepare the two slices for the _difference_ set operation.
    pub fn symmetric_difference(self) -> SymmetricDifference<'a, T> {
        SymmetricDifference::new(self.a, self.b)
    }
}
