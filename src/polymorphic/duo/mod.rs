use set::Set;

mod difference;

pub use self::difference::Difference;

/// Type used to make a set operation on two slices only.
#[derive(Copy, Clone)]
pub struct OpBuilder<'a, T: 'a, U: 'a, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    a: &'a Set<T>,
    b: &'a Set<U>,
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> OpBuilder<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    /// Construct a type with two slices.
    pub fn new(a: &'a Set<T>, b: &'a Set<U>, f: F, g: G) -> Self {
        Self { a, b, f, g }
    }

    // /// Prepare the two slices for the _union_ set operation.
    // pub fn union(self) -> Union<'a, T> {
    //     Union::new(self.a, self.b)
    // }

    // /// Prepare the two slices for the _intersection_ set operation.
    // pub fn intersection(self) -> Intersection<'a, T> {
    //     Intersection::new(self.a, self.b)
    // }

    /// Prepare the two slices for the _difference_ set operation.
    pub fn difference(self) -> Difference<'a, T, U, F, G, K> {
        Difference::new(self.a, self.b, self.f, self.g)
    }

    // /// Prepare the two slices for the _difference_ set operation.
    // pub fn symmetric_difference(self) -> SymmetricDifference<'a, T> {
    //     SymmetricDifference::new(self.a, self.b)
    // }
}
