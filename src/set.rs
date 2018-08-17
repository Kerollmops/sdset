//! All the methods and types associated to [`Set`]s.

use std::cmp::Ordering;
use std::ops::Deref;
use std::{error, fmt, mem};

/// Represent a slice which contains types that are sorted and deduplicated (akin to [`str`]).
///
/// This is an *unsized* type, meaning that it must always be used behind a
/// pointer like `&` or [`Box`]. For an owned version of this type,
/// see [`SetBuf`].
#[repr(C)] // TODO replace by repr(transparent)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Set<T>([T]);

impl<T> Set<T> {
    /// Construct a [`Set`] only if it is sorted and deduplicated.
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::Set;
    ///
    /// let slice = &[1, 2, 4, 6, 7];
    /// let set = Set::new(slice)?;
    ///
    /// // this slice is not sorted!
    /// let slice = &[1, 2, 4, 7, 6];
    /// let set = Set::new(slice);
    ///
    /// assert_eq!(set, Err(Error::NotSort));
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new(slice: &[T]) -> Result<&Self, Error>
    where T: Ord
    {
        is_sort_dedup(slice).map(|_| Self::new_unchecked(slice))
    }

    /// Construct a [`Set`] without checking it.
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::Set;
    ///
    /// // this slice is not sorted
    /// let slice = &[1, 2, 4, 7, 6];
    ///
    /// // but we can still create a Set, so be carreful!
    /// let set = Set::new_unchecked(slice);
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new_unchecked(slice: &[T]) -> &Self {
        unsafe { mem::transmute(slice) }
    }

    /// Construct the owning version of the [`Set`].
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::{Set, SetBuf};
    ///
    /// let set = Set::new(&[1, 2, 4, 6, 7])?;
    /// let setbuf: SetBuf<_> = set.to_set_buf();
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn to_set_buf(&self) -> SetBuf<T>
    where T: Clone
    {
        SetBuf(self.0.to_vec())
    }

    /// Return the slice "inside" of this [`Set`].
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::Set;
    ///
    /// let slice = &[1, 2, 4, 6, 7];
    /// let set = Set::new(slice)?;
    ///
    /// assert_eq!(set.as_slice(), slice);
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

impl<T> Deref for Set<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> AsRef<[T]> for Set<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> AsRef<Set<T>> for Set<T> {
    fn as_ref(&self) -> &Set<T> {
        self
    }
}

/// An owned, set (akin to [`String`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetBuf<T>(Vec<T>);

impl<T> SetBuf<T> {
    /// Construct a [`SetBuf`] only if it is sorted and deduplicated.
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::SetBuf;
    ///
    /// let vec = vec![1, 2, 4, 6, 7];
    /// let setbuf = SetBuf::new(vec)?;
    ///
    /// // this vec is not sorted!
    /// let vec = vec![1, 2, 4, 7, 6];
    /// let setbuf = SetBuf::new(vec);
    ///
    /// assert_eq!(setbuf, Err(Error::NotSort));
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new(vec: Vec<T>) -> Result<Self, Error>
    where T: Ord
    {
        is_sort_dedup(&vec).map(|_| SetBuf::new_unchecked(vec))
    }

    /// Construct a [`SetBuf`] without checking it.
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::SetBuf;
    ///
    /// // this vec is not sorted
    /// let vec = vec![1, 2, 4, 7, 6];
    ///
    /// // but we can still create a SetBuf, so be carreful!
    /// let setbuf = SetBuf::new_unchecked(vec);
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn new_unchecked(vec: Vec<T>) -> Self {
        SetBuf(vec)
    }

    /// Return the [`Set`] owned by this [`SetBuf`].
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::{Set, SetBuf};
    ///
    /// let vec = vec![1, 2, 4, 6, 7];
    /// let setbuf = SetBuf::new(vec.clone())?;
    ///
    /// let set = Set::new(&vec)?;
    /// assert_eq!(setbuf.as_set(), set);
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn as_set(&self) -> &Set<T> {
        Set::new_unchecked(self.0.as_slice())
    }

    /// Return the [`Vec`] inside by this [`SetBuf`].
    ///
    /// ```
    /// # use sdset::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use sdset::SetBuf;
    ///
    /// let vec = vec![1, 2, 4, 6, 7];
    /// let setbuf = SetBuf::new(vec)?;
    ///
    /// let vec = setbuf.into_vec();
    /// # Ok(()) }
    /// # try_main().unwrap();
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Deref for SetBuf<T> {
    type Target = Set<T>;

    fn deref(&self) -> &Self::Target {
        self.as_set()
    }
}

impl<T> AsRef<Set<T>> for SetBuf<T> {
    fn as_ref(&self) -> &Set<T> {
        self.as_set()
    }
}

impl<T> AsRef<[T]> for SetBuf<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

/// Represent the possible errors when creating a [`Set`].
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

/// The list of all [`Error`]s that can occur
/// while trying to convert a [`slice`](std::slice) to a [`Set`].
pub type Errors = Vec<Option<Error>>;

/// Construct a [`Vec`] of [`Set`]s only if all slices are sorted and deduplicated.
///
/// Otherwise returns the [`Vec`] given in parameter along with a [`Vec`] containing
/// the possible conversion errors of each slice.
///
/// # Examples
/// ```
/// use sdset::set::slices_into_sets;
///
/// let a = &[1, 2, 3, 4][..];
/// let b = &[1, 4, 6, 7];
/// let slices = vec![a, b];
///
/// let sets = slices_into_sets(slices).unwrap();
/// ```
pub fn slices_into_sets<T: Ord>(vec: Vec<&[T]>) -> Result<Vec<&Set<T>>, (Vec<&[T]>, Errors)> {
    let mut has_error = false;
    let mut errors = Errors::with_capacity(vec.len());
    for slice in &vec {
        let res = is_sort_dedup(slice).err();
        has_error = res.is_some();
        errors.push(res);
    }

    if has_error {
        return Err((vec, errors))
    }

    Ok(slices_into_sets_unchecked(vec))
}

/// Transmutes slices without checking them.
///
/// # Examples
/// ```
/// use sdset::set::slices_into_sets_unchecked;
///
/// // these slices are not sorted!
/// let a = &[1, 6, 4][..];
/// let b = &[1, 6, 1];
/// let slices = vec![a, b];
///
/// // but we can still create a Vec of Sets, so be carreful!
/// let sets = slices_into_sets_unchecked(slices);
/// ```
pub fn slices_into_sets_unchecked<T>(vec: Vec<&[T]>) -> Vec<&Set<T>> {
    unsafe { mem::transmute(vec) }
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
