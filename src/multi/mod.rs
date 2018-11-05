//! Contains the types to make set operations on any given number of slices.
//!
//! # Examples
//! ```
//! # use sdset::Error;
//! # fn try_main() -> Result<(), Error> {
//! use sdset::multi::OpBuilder;
//! use sdset::{SetOperation, Set, SetBuf};
//!
//! let a = Set::new(&[1, 2, 4])?;
//! let b = Set::new(&[2, 3, 5, 7])?;
//! let c = Set::new(&[4, 6, 7])?;
//!
//! let op = OpBuilder::from_vec(vec![a, b, c]).union();
//!
//! let res: SetBuf<i32> = op.into_set_buf();
//! assert_eq!(&res[..], &[1, 2, 3, 4, 5, 6, 7]);
//! # Ok(()) }
//! # try_main().unwrap();
//! ```

use crate::set::Set;

mod union;
mod intersection;
mod difference;
mod difference_by_key;
mod symmetric_difference;

pub use self::union::Union;
pub use self::intersection::Intersection;
pub use self::difference::Difference;
pub use self::difference_by_key::DifferenceByKey;
pub use self::symmetric_difference::SymmetricDifference;

/// Type used to acquire any number of slices
/// and make a set operation on these slices.
#[derive(Clone)]
pub struct OpBuilder<'a, T: 'a> {
    slices: Vec<&'a Set<T>>,
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
    pub fn from_vec(slices: Vec<&'a Set<T>>) -> Self {
        Self { slices }
    }

    /// Reserve additional space for the underlying vec.
    pub fn reserve(&mut self, additional: usize) {
        self.slices.reserve(additional);
    }

    /// Add a new set that will be used for the future set operation
    /// and consume and return the type.
    pub fn add(mut self, set: &'a Set<T>) -> Self {
        self.push(set);
        self
    }

    /// Push a new set that will be used for the future set operation.
    pub fn push(&mut self, set: &'a Set<T>) {
        self.slices.push(set);
    }

    /// Prepare the slices for the _union_ set operation.
    pub fn union(self) -> Union<'a, T> {
        Union::new(self.slices)
    }

    /// Prepare the slices for the _intersection_ set operation.
    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new(self.slices)
    }

    /// Prepare the slices for the _difference_ set operation.
    pub fn difference(self) -> Difference<'a, T> {
        Difference::new(self.slices)
    }

    /// Prepare the slices for the _symmetric difference_ set operation.
    pub fn symmetric_difference(self) -> SymmetricDifference<'a, T> {
        SymmetricDifference::new(self.slices)
    }
}

/// Type used to make a set operation on two slices of different types.
///
/// The two functions are used to generate a key that will be used to
/// make the set operation and correlate the two slices values.
#[derive(Clone)]
pub struct OpBuilderByKey<'a, T: 'a, U: 'a, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    base: &'a Set<T>,
    others: Vec<&'a Set<U>>,
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> OpBuilderByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    /// Construct a type with two slices.
    pub fn new(base: &'a Set<T>, f: F, g: G) -> Self {
        Self { base, others: Vec::new(), f, g }
    }

    /// Construct it with the content of the given slice.
    ///
    /// Note that no other allocation than the one of the vec given
    /// in parameter is needed for the construction.
    pub fn from_vec(base: &'a Set<T>, others: Vec<&'a Set<U>>, f: F, g: G) -> Self {
        Self { base, others, f, g }
    }

    /// Push a new set that will be used for the future set operation.
    pub fn push(&mut self, set: &'a Set<U>) {
        self.others.push(set);
    }

    /// Add a new set that will be used for the future set operation
    /// and consume and return the type.
    pub fn add(mut self, set: &'a Set<U>) -> Self {
        self.push(set);
        self
    }

    /// Prepare the two slices for the _difference_ set operation.
    pub fn difference(self) -> DifferenceByKey<'a, T, U, F, G, K> {
        DifferenceByKey::new(self.base, self.others, self.f, self.g)
    }
}
