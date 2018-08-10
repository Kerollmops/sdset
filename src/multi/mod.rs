use std::mem;
use sort_dedup::SortDedup;

mod union;
mod intersection;
mod difference;

pub use self::union::Union;
pub use self::intersection::Intersection;
pub use self::difference::Difference;

pub struct OpBuilder<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> OpBuilder<'a, T> {
    pub fn new() -> Self {
        Self { slices: Vec::new() }
    }

    pub fn from_vec(slices: Vec<SortDedup<'a, T>>) -> Self {
        // the SortDedup type is marked as transparent
        // so it is safe to transmute it to the underlying slice
        // transmuting here is done to avoid doing a useless allocation
        Self::from_vec_unchecked(unsafe { mem::transmute(slices) })
    }

    pub fn from_vec_unchecked(slices: Vec<&'a [T]>) -> Self {
        Self { slices }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { slices: Vec::with_capacity(capacity) }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.slices.reserve(additional);
    }

    pub fn add(mut self, slice: &'a [T]) -> Self {
        self.push(slice);
        self
    }

    pub fn push(&mut self, slice: &'a [T]) {
        self.slices.push(slice);
    }

    pub fn union(self) -> Union<'a, T> {
        Union::new_unchecked(self.slices)
    }

    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new_unchecked(self.slices)
    }

    pub fn difference(self) -> Difference<'a, T> {
        Difference::new_unchecked(self.slices)
    }
}
