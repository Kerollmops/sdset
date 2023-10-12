use std::cmp::Ordering;
use crate::set::Set;
use crate::{SetOperation, exponential_search_by};

/// Represent the _intersection_ set operation that will be applied to two slices of different types.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::duo::OpBuilderByKey;
/// use sdset::{SetOperation, Set, SetBuf};
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// struct Foo { a: i32, b: u8 }
///
/// let a = Set::new(&[
///     Foo { a: 1, b: 6 },
///     Foo { a: 2, b: 7 },
///     Foo { a: 2, b: 8 },
///     Foo { a: 3, b: 8 },
///     Foo { a: 4, b: 9 },
///     Foo { a: 6, b: 10 },
///     Foo { a: 7, b: 10 },
/// ])?;
/// let b = Set::new(&[2, 3, 4, 5, 6, 7])?;
///
/// // Return the field of Foo that will be used for comparison
/// let f = |x: &Foo| x.a;
///
/// // directly use the i32 for comparison
/// let g = |x: &i32| *x;
///
/// let op = OpBuilderByKey::new(a, b, f, g).intersection();
///
/// let res: SetBuf<Foo> = op.into_set_buf();
/// let expected = &[
///     Foo { a: 2, b: 7 },
///     Foo { a: 2, b: 8 },
///     Foo { a: 3, b: 8 },
///     Foo { a: 4, b: 9 },
///     Foo { a: 6, b: 10 },
///     Foo { a: 7, b: 10 },
/// ][..];
///
/// assert_eq!(res.as_slice(), expected);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct IntersectionByKey<'a, T: 'a, U: 'a, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    a: &'a [T],
    b: &'a [U],
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> IntersectionByKey<'a, T, U, F, G, K>
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

impl<'a, T, U, F, G, K> IntersectionByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord + std::fmt::Debug,
    T: std::fmt::Debug
{
    #[inline]
    fn extend_vec<X, E>(mut self, output: &mut Vec<X>, extend: E)
    where E: Fn(&mut Vec<X>, &'a [T]),
    {
        while !self.a.is_empty() && !self.b.is_empty() {
            let keyb = (self.g)(&self.b[0]);

            // skip to the key to take
            let off = exponential_search_by(self.a, |a| {
                match (self.f)(a).cmp(&keyb) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => Ordering::Greater,
                    Ordering::Greater => Ordering::Greater,
                }
            }).unwrap_err();

            self.a = &self.a[off..];

            // position after the last key to take
            let off = exponential_search_by(self.a, |a| {
                match (self.f)(a).cmp(&keyb) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                }
            }).unwrap_err();

            extend(output, &self.a[..off]);
            self.a = &self.a[off..];

            let offb = self.b.iter().take_while(|b| (self.g)(b) == keyb).count();
            self.b = &self.b[offb..];
        }
    }
}

impl<'a, T, U, F, G, K> SetOperation<T> for IntersectionByKey<'a, T, U, F, G, K>
where T: Clone + std::fmt::Debug,
      F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord + std::fmt::Debug
{
    fn extend_vec(self, output: &mut Vec<T>) {
        self.extend_vec(output, Vec::extend_from_slice)
    }
}

impl<'a, T, U, F, G, K> SetOperation<&'a T> for IntersectionByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord + std::fmt::Debug,
      T: std::fmt::Debug,
{
    fn extend_vec(self, output: &mut Vec<&'a T>) {
        self.extend_vec(output, Extend::extend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::set::{sort_dedup_vec, SetBuf};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Foo {
        a: i32,
        b: i8,
    }

    #[test]
    fn two_slices() {
        let a = &[
            Foo { a: 1, b: 23 },
            Foo { a: 2, b: 23 },
            Foo { a: 3, b: 23 },
        ];
        let b = &[2, 3, 4];
        let f = |x: &Foo| x.a;
        let g = |x: &i32| *x;

        let intersection_: SetBuf<Foo> = IntersectionByKey { a, b, f, g }.into_set_buf();
        assert_eq!(&intersection_[..], &[Foo { a: 2, b: 23 }, Foo { a: 3, b: 23 }]);
    }

    quickcheck! {
        fn qc_intersection(a: Vec<i32>, b: Vec<i32>) -> bool {
            use std::collections::BTreeSet;
            use std::iter::FromIterator;

            let mut a = a;
            let mut b = b;
            let f = Clone::clone;
            let g = Clone::clone;

            sort_dedup_vec(&mut a);
            sort_dedup_vec(&mut b);

            let x: SetBuf<i32> = IntersectionByKey { a: &a, b: &b, f, g }.into_set_buf();

            let a = BTreeSet::from_iter(a);
            let b = BTreeSet::from_iter(b);
            let y = a.intersection(&b);
            let y: Vec<_> = y.cloned().collect();

            x.as_slice() == y.as_slice()
        }
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use super::*;
    use self::test::Bencher;
    use crate::set::SetBuf;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Foo {
        a: i32,
        b: i8,
    }

    impl Foo {
        fn new(a: i32) -> Foo {
            Foo { a, b: 0 }
        }
    }

    #[bench]
    fn two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (1..101).collect();
        let f = |x: &Foo| x.a;
        let g = Clone::clone;

        bench.iter(|| {
            let op = IntersectionByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (51..151).collect();
        let f = |x: &Foo| x.a;
        let g = Clone::clone;

        bench.iter(|| {
            let op = IntersectionByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (100..200).collect();
        let f = |x: &Foo| x.a;
        let g = Clone::clone;

        bench.iter(|| {
            let op = IntersectionByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }
}
