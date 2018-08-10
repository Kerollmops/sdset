use std::cmp::Ordering;
use std::{error, fmt};

/// Represent a slice which contains types that are sorted and deduplicated.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SortDedup<'a, T: 'a>(&'a [T]);

/// Represent the possible errors when creating a [`SortDedup`] slice.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// Define that a slice is not sorted.
    NotSort,
    /// Define that a slice is not deduplicated.
    NotDedup,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let desc = match self {
            Error::NotSort => "the given slice is not sorted.",
            Error::NotDedup => "the given slice is not deduplicated.",
        };
        f.write_str(desc)
    }
}

impl error::Error for Error {}

impl<'a, T> SortDedup<'a, T> {
    /// Construct the type only if it is sorted and deduplicated.
    ///
    /// ```
    /// # use setiter::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use setiter::SortDedup;
    ///
    /// let slice = &[1, 2, 4, 6, 7];
    /// let sd = SortDedup::new(slice)?;
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new(slice: &'a [T]) -> Result<Self, Error>
    where T: Ord
    {
        is_sort_dedup(slice).map(|_| SortDedup(slice))
    }

    /// Construct the type without checking if the slice.
    ///
    /// ```
    /// # use setiter::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use setiter::SortDedup;
    ///
    /// // this slice is not sorted
    /// let slice = &[1, 2, 4, 7, 6];
    ///
    /// // but we can still create one, so be carreful!
    /// let sd = SortDedup::new_unchecked(slice);
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new_unchecked(slice: &'a [T]) -> Self {
        SortDedup(slice)
    }

    /// Transform the underlying slice.
    pub fn into_slice(self) -> &'a [T] {
        self.0
    }
}

/// Sort and dedup the vec given in parameter.
pub fn sort_dedup_vec<T: Ord>(vec: &mut Vec<T>) {
    vec.sort_unstable();
    vec.dedup();
}

/// Returns an error if the slice is not sorted nor deduplicated, returns `()` if it is.
pub fn is_sort_dedup<T: Ord>(slice: &[T]) -> Result<(), Error> {
    for pair in slice.windows(2) {
        match pair[0].cmp(&pair[1]) {
            Ordering::Less => (),
            Ordering::Equal => return Err(Error::NotDedup),
            Ordering::Greater => return Err(Error::NotSort),
        }
    }
    Ok(())
}
