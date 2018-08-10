use sort_dedup::SortDedup;

mod union;
mod difference;
mod intersection;

pub use self::union::Union;
pub use self::difference::Difference;
pub use self::intersection::Intersection;

pub struct OpBuilder<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> OpBuilder<'a, T> {
    pub fn new(a: SortDedup<'a, T>, b: SortDedup<'a, T>) -> Self {
        Self::new_unchecked(a.into_slice(), b.into_slice())
    }

    pub fn new_unchecked(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b }
    }

    pub fn union(self) -> Union<'a, T> {
        Union::new_unchecked(self.a, self.b)
    }

    pub fn intersection(self) -> Intersection<'a, T> {
        Intersection::new_unchecked(self.a, self.b)
    }

    pub fn difference(self) -> Difference<'a, T> {
        Difference::new_unchecked(self.a, self.b)
    }
}
