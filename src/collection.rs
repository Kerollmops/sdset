use std::hash::Hash;
use std::collections::{HashSet, BTreeSet};

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
/// let mut counter = Counter::default();
/// SetOperation::<i32>::extend_collection(op, &mut counter);
///
/// assert_eq!(counter.0, 7);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Default)]
pub struct Counter(pub usize);

impl Counter {
    /// Create a new [`Counter`] initialized with 0;
    pub fn new() -> Counter {
        Counter::default()
    }
}

impl<T> Collection<T> for Counter {
    fn push(&mut self, _elem: T) {
        self.0 = self.0.saturating_add(1);
    }

    fn extend_from_slice(&mut self, elems: &[T]) where T: Clone {
        self.0 = self.0.saturating_add(elems.len());
    }

    fn extend<I>(&mut self, elems: I) where I: IntoIterator<Item=T> {
        self.0 = self.0.saturating_add(elems.into_iter().count());
    }
}
