mod union;

pub use self::union::Union;

pub struct OpBuilder<'a, T: 'a> {
    a: &'a [T],
    b: &'a [T],
}

impl<'a, T> OpBuilder<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b }
    }

    pub fn union(self) -> Union<'a, T> {
        Union::new(self.a, self.b)
    }

    // pub fn intersection(self) -> Intersection<'a, T> {
    //     Intersection::new(self.slices)
    // }

    // pub fn difference(self) -> Difference<'a, T> {
    //     Difference::new(self.slices)
    // }
}
