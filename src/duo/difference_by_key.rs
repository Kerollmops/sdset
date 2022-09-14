use crate::set::Set;
use crate::{exponential_offset_ge_by_key, SetOperation, Collection};

/// Represent the _difference_ set operation that will be applied to two slices of different types.
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
///     Foo{ a: 1, b: 6 },
///     Foo{ a: 1, b: 7 },
///     Foo{ a: 1, b: 8 },
///     Foo{ a: 2, b: 9 },
///     Foo{ a: 2, b: 10 },
///     Foo{ a: 3, b: 10 },
/// ])?;
/// let b = Set::new(&[1, 3, 4, 5]).unwrap();
///
/// // Return the field of Foo that will be used for comparison
/// let f = |x: &Foo| x.a;
///
/// // directly use the i32 for comparison
/// let g = |x: &i32| *x;
///
/// let op = OpBuilderByKey::new(a, b, f, g).difference();
/// let res: SetBuf<Foo> = op.into_set_buf();
///
/// assert_eq!(res.as_slice(), &[Foo{ a: 2, b: 9 }, Foo{ a: 2, b: 10 }][..]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Copy, Clone)]
pub struct DifferenceByKey<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
    G: Fn(&U) -> K,
    K: Ord,
{
    a: &'a [T],
    b: &'a [U],
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> DifferenceByKey<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
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

impl<'a, T, U, F, G, K> DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_collection<C, X, E>(mut self, output: &mut C, extend: E) -> Result<(), C::Error>
    where C: Collection<X>,
          E: Fn(&mut C, &'a [T]) -> Result<(), C::Error>,
    {
        while let Some(first_a) = self.a.first().map(|x| (self.f)(x)) {
            self.b = exponential_offset_ge_by_key(self.b, &first_a, &self.g);

            match self.b.first().map(|x| (self.g)(x)) {
                Some(min) => {
                    if min == first_a {
                        self.a = &self.a[1..];
                        // cannot advance b since we support duplicate relations
                    } else {
                        let off = self.a.iter().take_while(|&x| (self.f)(x) < min).count();
                        extend(output, &self.a[..off])?;

                        self.a = &self.a[off..]
                    }
                },
                None => {
                    extend(output, self.a)?;
                    break;
                },
            }
        }
        Ok(())
    }

    fn iter(&'a self) -> DifferenceByKeyIter<'a, T, U, F, G, K>
    {
        DifferenceByKeyIter {
            a: self.a,
            b: self.b,
            f: &self.f,
            g: &self.g
        }
    }
}

impl<'a, T, U, F, G, K> SetOperation<T> for DifferenceByKey<'a, T, U, F, G, K>
where T: Clone,
      F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<T>,
    {
        self.extend_collection(output, Collection::extend_from_slice)
    }
}

impl<'a, T, U, F, G, K> SetOperation<&'a T> for DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    fn extend_collection<C>(self, output: &mut C) -> Result<(), C::Error>
    where C: Collection<&'a T>,
    {
        self.extend_collection(output, Collection::extend)
    }
}

// This version of IntoIterator takes references to the functions (f/g).
// The separate structs are required to not break the public API which takes the functions
//   by value instead of by reference, and doesn't require them to implement Copy/Clone.
impl<'a, T, U, F, G, K> IntoIterator for &'a DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    type Item = &'a T;
    type IntoIter = DifferenceByKeyIter<'a, T, U, F, G, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct DifferenceByKeyIter<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
    G: Fn(&U) -> K,
    K: Ord,
{
    a: &'a [T],
    b: &'a [U],
    f: &'a F,
    g: &'a G,
}

impl<'a, T, U, F, G, K> Iterator for DifferenceByKeyIter<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
    G: Fn(&U) -> K,
    K: Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.a.is_empty() {
                return None;
            }
            let first_a = (self.f)(&self.a[0]);
            self.b = exponential_offset_ge_by_key(self.b, &first_a, &self.g);
            if self.b.is_empty() {
                let result = &self.a[0];
                self.a = &self.a[1..];
                return Some(result);
            }
            if first_a == (self.g)(&self.b[0]) {
                self.a = &self.a[1..];
                // cannot advance b since we support duplicate relations
                continue;
            } else { // b > a
                let result = &self.a[0];
                self.a = &self.a[1..];
                return Some(result);
            }
        }
    }
}

// This version of IntoIterator moves the contents of self into the iterator.
// Therefore the iterator owns the functions (f/g).
// The separate structs are required to not break the public API which takes the functions
//   by value instead of by reference, and doesn't require them to implement Copy/Clone.
impl<'a, T, U, F, G, K> IntoIterator for DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K + 'a,
      G: Fn(&U) -> K + 'a,
      K: Ord + 'a,
{
    type Item = &'a T;
    type IntoIter = DifferenceByKeyIterOwning<'a, T, U, F, G, K>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            a: self.a,
            b: self.b,
            f: self.f,
            g: self.g
        }
    }
}

pub struct DifferenceByKeyIterOwning<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
    G: Fn(&U) -> K,
    K: Ord,
{
    a: &'a [T],
    b: &'a [U],
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> Iterator for DifferenceByKeyIterOwning<'a, T, U, F, G, K>
where
    T: 'a,
    U: 'a,
    F: Fn(&T) -> K,
    G: Fn(&U) -> K,
    K: Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.a.is_empty() {
                return None;
            }
            let first_a = (self.f)(&self.a[0]);
            self.b = exponential_offset_ge_by_key(self.b, &first_a, &self.g);
            if self.b.is_empty() {
                let result = &self.a[0];
                self.a = &self.a[1..];
                return Some(result);
            }
            if first_a == (self.g)(&self.b[0]) {
                self.a = &self.a[1..];
                // cannot advance b since we support duplicate relations
                continue;
            } else { // b > a
                let result = &self.a[0];
                self.a = &self.a[1..];
                return Some(result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod set_to_set {
        use super::super::*;
        use crate::set::{sort_dedup_vec, SetBuf};

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Foo {
            a: i32,
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

            let difference: SetBuf<Foo> = DifferenceByKey::new(a, b, |x| x.a, |&x| x).into_set_buf();

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

            let difference: SetBuf<Foo> = DifferenceByKey::new(a, b, |x| x.a, |&x| x).into_set_buf();

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

            let difference: SetBuf<Foo> = DifferenceByKey::new(a, b, |x| x.a, |&x| x).into_set_buf();

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
                    let difference = DifferenceByKey { a: &a, b: &b, f: |&x| x, g: |&x| x as i32 };
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

    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Foo {
            a: i32,
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
            let difference = DifferenceByKey::new(a, b, |x| x.a, |&x| x);

            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert!(diff_ref.is_empty());

            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert!(diff_own.is_empty());
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
            
            let difference = DifferenceByKey::new(a, b, |x| x.a, |&x| x);

            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert!(diff_ref.is_empty());

            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert!(diff_own.is_empty());
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

            let difference = DifferenceByKey::new(a, b, |x| x.a, |&x| x);

            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(diff_ref.as_slice(), &[
                Foo{ a: 2, b: 9  },
                Foo{ a: 2, b: 10 },
            ][..]);

            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(diff_own.as_slice(), &[
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

                let x: Vec<i32> = {
                    let difference = DifferenceByKey { a: &a, b: &b, f: |&x| x, g: |&x| x as i32 };
                    difference.into_iter().cloned().collect()
                };

                let a = BTreeSet::from_iter(a);
                let b = BTreeSet::from_iter(b.into_iter().map(|x| x as i32));
                let y = a.difference(&b);
                let y: Vec<_> = y.cloned().collect();

                x.as_slice() == y.as_slice()
            }
        }
    }
}


#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use super::*;
    use self::test::Bencher;
    use crate::set::SetBuf;

    #[derive(Debug, Clone)]
    pub struct Foo {
        a: i32,
        b: u8
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
        let g = |x: &i32| *x;

        bench.iter(|| {
            let op = DifferenceByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (51..151).collect();
        let f = |x: &Foo| x.a;
        let g = |x: &i32| *x;

        bench.iter(|| {
            let op = DifferenceByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (100..200).collect();
        let f = |x: &Foo| x.a;
        let g = |x: &i32| *x;

        bench.iter(|| {
            let op = DifferenceByKey { a: &a, b: &b, f, g };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }
}
