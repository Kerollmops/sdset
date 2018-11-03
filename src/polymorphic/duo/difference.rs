use set::Set;
use SetOperation;

#[derive(Copy, Clone)]
pub struct Difference<'a, T: 'a, U: 'a, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    a: &'a [T],
    b: &'a [U],
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> Difference<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(a: &'a Set<T>, b: &'a Set<U>, f: F, g: G) -> Self {
        Self {
            a: a.as_slice(),
            b: b.as_slice(),
            f: f,
            g: g,
        }
    }
}

#[inline]
fn offset_ge<'a, 'b, T: 'a, F, K>(slice: &'a [T], f: F, elem: &'b K) -> &'a [T]
where F: Fn(&'a T) -> K,
      K: PartialOrd,
{
    match slice.iter().position(|x| &f(x) >= elem) {
        Some(pos) => &slice[pos..],
        None => &[],
    }
}

impl<'a, T, U, F, G, K> Difference<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_vec<X, E>(mut self, output: &mut Vec<X>, extend: E)
    where E: Fn(&mut Vec<X>, &'a [T]),
    {
        while let Some(first) = self.a.first().map(|x| (self.f)(x)) {
            self.b = offset_ge(self.b, &self.g, &first);

            match self.b.first().map(|x| (self.g)(x)) {
                Some(min) => {
                    if min == first {
                        self.a = offset_ge(&self.a[1..], &self.f, &min)
                    } else {
                        let off = self.a.iter().take_while(|&x| (self.f)(x) < min).count();
                        extend(output, &self.a[..off]);

                        self.a = &self.a[off..]
                    }
                },
                None => {
                    extend(output, self.a);
                    break;
                },
            }
        }
    }
}

impl<'a, T, U, F, G, K> SetOperation<T> for Difference<'a, T, U, F, G, K>
where T: Clone,
      F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice)
    }
}

impl<'a, T, U, F, G, K> SetOperation<&'a T> for Difference<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use set::{sort_dedup_vec, SetBuf};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Foo {
        a: u32,
        b: i8,
    }

    #[test]
    fn difference_empty_no_duplicates() {
        let a = Set::new_unchecked(&[
            Foo{ a: 1, b: 8 },
            Foo{ a: 2, b: 9 },
            Foo{ a: 3, b: 10 },
            Foo{ a: 4, b: 11 },
            Foo{ a: 5, b: 12 },
        ]);
        let b = Set::new(&[1, 2, 3, 4, 5]).unwrap();

        let difference: SetBuf<Foo> = Difference::new(a, b, |x| x.a, |&x| x).into_set_buf();

        assert!(difference.is_empty());
    }

    #[test]
    fn difference_empty_duplicate_relations() {
        let a = Set::new_unchecked(&[
            Foo{ a: 1, b: 6 },
            Foo{ a: 1, b: 7 },
            Foo{ a: 1, b: 8 },
            Foo{ a: 2, b: 9 },
            Foo{ a: 2, b: 10 },
        ]);
        let b = Set::new(&[1, 2, 3, 4, 5]).unwrap();

        let difference: SetBuf<Foo> = Difference::new(a, b, |x| x.a, |&x| x).into_set_buf();

        assert!(difference.is_empty());
    }

    #[test]
    fn difference_non_empty_duplicate_relations() {
        let a = Set::new_unchecked(&[
            Foo{ a: 1, b: 6 },
            Foo{ a: 1, b: 7 },
            Foo{ a: 1, b: 8 },
            Foo{ a: 2, b: 9 },
            Foo{ a: 2, b: 10 },
        ]);
        let b = Set::new(&[1, 3, 4, 5]).unwrap();

        let difference: SetBuf<Foo> = Difference::new(a, b, |x| x.a, |&x| x).into_set_buf();

        assert_eq!(difference.as_slice(), &[
            Foo{ a: 2, b: 9  },
            Foo{ a: 2, b: 10 },
        ][..]);
    }

    quickcheck! {
        fn qc_difference(a: Vec<i32>, b: Vec<i64>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;

            sort_dedup_vec(&mut a);
            sort_dedup_vec(&mut b);

            let x: SetBuf<i32> = {
                let difference = Difference { a: &a, b: &b, f: |&x| x, g: |&x| x as i32 };
                difference.into_set_buf()
            };

            let a = BTreeSet::from_iter(a);
            let b = BTreeSet::from_iter(b.into_iter().map(|x| x as i32));
            let y = a.difference(&b);
            let y: Vec<_> = y.cloned().collect();

            x.as_slice() == y.as_slice()
        }
    }
}
