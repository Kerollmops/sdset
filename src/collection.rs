use std::hash::Hash;
use std::collections::{HashSet, BTreeSet};
use std::marker;

/// This trait is meant to abstract any kind of collection
/// (i.e. [`Vec`], [`HashSet`]).
///
/// This is particularly helpful when you want particular behavior
/// when inserting elements, the [`Counter`] struct is a good example
/// of a custom implementation of the [`Collection`] trait, it is used to only
/// count the number of elements of a set operation.
pub trait Collection<T> {

    /// Insert one element into the collection.
    fn push(&mut self, elem: T);

    /// Extend the collection by cloning the elements.
    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone;

    /// Extend the collection by inserting the elements from the given [`Iterator`].
    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T>;

    /// Reserve enough space in the collection for `size` elements.
    fn reserve(&mut self, _size: usize) { }
}

impl<T> Collection<T> for Vec<T> {
    fn push(&mut self, elem: T) {
        Vec::push(self, elem);
    }

    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone {
        Vec::extend_from_slice(self, elems);
    }

    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T> {
        Extend::extend(self, elems);
    }

    fn reserve(&mut self, size: usize) {
        Vec::reserve(self, size);
    }
}

impl<T: Hash + Eq> Collection<T> for HashSet<T> {
    fn push(&mut self, elem: T) {
        HashSet::insert(self, elem);
    }

    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone {
        Collection::extend(self, elems.iter().cloned());
    }

    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T> {
        Extend::extend(self, elems);
    }

    fn reserve(&mut self, size: usize) {
        HashSet::reserve(self, size);
    }
}

impl<T: Ord> Collection<T> for BTreeSet<T> {
    fn push(&mut self, elem: T) {
        BTreeSet::insert(self, elem);
    }

    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone {
        Collection::extend(self, elems.iter().cloned());
    }

    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T> {
        Extend::extend(self, elems);
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
    fn push(&mut self, _elem: T) {
        self.count = self.count.saturating_add(1);
    }

    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone {
        self.count = self.count.saturating_add(elems.len());
    }

    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T> {
        self.count = self.count.saturating_add(elems.into_iter().count());
    }
}
