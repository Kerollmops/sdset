use std::hash::Hash;
use std::collections::{HashSet, BTreeSet};
use std::marker;
use std::convert::Infallible;

/// This trait is meant to abstract any kind of collection
/// (i.e. [`Vec`], [`HashSet`]).
///
/// This is particularly helpful when you want particular behavior
/// when inserting elements, the [`Counter`] struct is a good example
/// of a custom implementation of the [`Collection`] trait, it is used to only
/// count the number of elements of a set operation.
pub trait Collection<T> {

    /// Error type associated with the [`Collection`].
    type Error;

    /// Insert one element into the collection.
    fn push(&mut self, elem: T) -> Result<(), Self::Error>;

    /// Extend the collection by cloning the elements.
    fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), Self::Error>
    where T: Clone;

    /// Extend the collection by inserting the elements from the given [`Iterator`].
    fn extend<I>(&mut self, elems: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item=T>;

    /// Reserve enough space in the collection for `size` elements.
    fn reserve(&mut self, _size: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T> Collection<T> for Vec<T> {

    type Error = Infallible;

    fn push(&mut self, elem: T) -> Result<(), Self::Error> {
        Vec::push(self, elem);
        Ok(())
    }

    fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), Self::Error>
    where T: Clone
    {
        Vec::extend_from_slice(self, elems);
        Ok(())
    }

    fn extend<I>(&mut self, elems: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item=T>
    {
        Extend::extend(self, elems);
        Ok(())
    }

    fn reserve(&mut self, size: usize) -> Result<(), Self::Error> {
        Vec::reserve(self, size);
        Ok(())
    }
}

impl<T: Hash + Eq> Collection<T> for HashSet<T> {

    type Error = Infallible;

    fn push(&mut self, elem: T) -> Result<(), Self::Error> {
        HashSet::insert(self, elem);
        Ok(())
    }

    fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), Self::Error>
    where T: Clone
    {
        Collection::extend(self, elems.iter().cloned())
    }

    fn extend<I>(&mut self, elems: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item=T>
    {
        Extend::extend(self, elems);
        Ok(())
    }

    fn reserve(&mut self, size: usize) -> Result<(), Self::Error> {
        HashSet::reserve(self, size);
        Ok(())
    }
}

impl<T: Ord> Collection<T> for BTreeSet<T> {

    type Error = Infallible;

    fn push(&mut self, elem: T)  -> Result<(), Self::Error> {
        BTreeSet::insert(self, elem);
        Ok(())
    }

    fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), Self::Error>
    where T: Clone
    {
        Collection::extend(self, elems.iter().cloned())
    }

    fn extend<I>(&mut self, elems: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item=T>
    {
        Extend::extend(self, elems);
        Ok(())
    }
}

/// A [`Collection`] that only counts the final size of a set operation.
///
/// It is meant to be used to avoid unecessary allocations.
///
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::duo::OpBuilder;
/// use sdset::{SetOperation, Set, SetBuf, Counter};
///
/// let a = Set::new(&[1, 2, 4, 6, 7])?;
/// let b = Set::new(&[2, 3, 4, 5, 6, 7])?;
///
/// let op = OpBuilder::new(a, b).union();
///
/// let mut counter = Counter::<i32>::default();
/// op.extend_collection(&mut counter);
///
/// assert_eq!(counter.get(), 7);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```

pub struct Counter<T> {
    count: usize,
    _phantom: marker::PhantomData<T>,
}

impl<T> Default for Counter<T> {
    fn default() -> Counter<T> {
        Counter {
            count: 0,
            _phantom: marker::PhantomData,
        }
    }
}

impl<T> Counter<T> {
    /// Create a new [`Counter`] initialized with 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the count for the [`Operation`].
    pub fn get(&self) -> usize {
        self.count
    }
}

impl<T> Collection<T> for Counter<T> {

    type Error = Infallible;

    fn push(&mut self, _elem: T) -> Result<(), Self::Error> {
        self.count = self.count.saturating_add(1);
        Ok(())
    }

    fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), Self::Error>
    where T: Clone
    {
        self.count = self.count.saturating_add(elems.len());
        Ok(())
    }

    fn extend<I>(&mut self, elems: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item=T>
    {
        self.count = self.count.saturating_add(elems.into_iter().count());
        Ok(())
    }
}
