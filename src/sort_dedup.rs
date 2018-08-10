use std::cmp::Ordering;
use std::{error, fmt};

/// Represent a slice which contains types that are sorted and deduplicated.
///
/// ```
/// # use setiter::Error;
/// # fn try_main() -> Result<(), Error> {
/// use setiter::SortDedup;
///
/// let slice = &[1, 2, 4, 6, 7];
/// let sd = SortDedup::new(slice)?;
/// # Ok(())
/// # }
/// # try_main().unwrap();
/// ```
#[repr(transparent)]
pub struct SortDedup<'a, T: 'a>(&'a [T]);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    NotSort,
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
    pub fn new(slice: &'a [T]) -> Result<Self, Error>
    where T: Ord
    {
        is_sort_dedup(slice).map(|_| SortDedup(slice))
    }

    pub fn new_unchecked(slice: &'a [T]) -> Self {
        SortDedup(slice)
    }

    pub fn into_slice(self) -> &'a [T] {
        self.0
    }
}

/// **Sort** and **dedup** the vec given in parameter.
pub fn sort_dedup_vec<T: Ord>(vec: &mut Vec<T>) {
    vec.sort_unstable();
    vec.dedup();
}

fn is_sort_dedup<T: Ord>(slice: &[T]) -> Result<(), Error> {
    for pair in slice.windows(2) {
        match pair[0].cmp(&pair[1]) {
            Ordering::Less => (),
            Ordering::Equal => return Err(Error::NotDedup),
            Ordering::Greater => return Err(Error::NotSort),
        }
    }
    Ok(())
}
