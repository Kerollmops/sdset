use std::cmp;
use crate::set::{Set, vec_sets_into_slices};
use crate::{SetOperation, Collection, exponential_offset_ge_by_key};

/// Represent the _difference_ set operation that will be applied to multiple slices
/// of two different types.
///
/// # Examples
/// ```
/// # use sdset::Error;
/// # fn try_main() -> Result<(), Error> {
/// use sdset::multi::OpBuilderByKey;
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
///     Foo{ a: 4, b: 10 },
/// ])?;
/// let b = Set::new(&[2, 3, 5, 7])?;
/// let c = Set::new(&[4, 6, 7])?;
///
/// // Return the field of Foo that will be used for comparison
/// let f = |x: &Foo| x.a;
///
/// // directly use the i32 for comparison
/// let g = |x: &i32| *x;
///
/// let op = OpBuilderByKey::from_vec(a, vec![b, c], f, g).difference();
/// let res: SetBuf<Foo> = op.into_set_buf();
///
/// assert_eq!(&res[..], &[Foo{ a: 1, b: 6 }, Foo{ a: 1, b: 7 }, Foo{ a: 1, b: 8 }]);
/// # Ok(()) }
/// # try_main().unwrap();
/// ```
#[derive(Clone)]
pub struct DifferenceByKey<'a, T: 'a, U: 'a, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    base: &'a [T],
    others: Vec<&'a [U]>,
    f: F,
    g: G,
}

impl<'a, T, U, F, G, K> DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    /// Construct one with slices checked to be sorted and deduplicated.
    pub fn new(base: &'a Set<T>, others: Vec<&'a Set<U>>, f: F, g: G) -> Self {
        Self {
            base: base.as_slice(),
            others: vec_sets_into_slices(others),
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
        while let Some(first) = self.base.first().map(|x| (self.f)(x)) {
            let mut minimum = None;
            for slice in self.others.iter_mut() {
                *slice = exponential_offset_ge_by_key(slice, &first, &self.g);

                let first = match slice.first() {
                    Some(first) => Some((self.g)(first)),
                    None => None,
                };

                minimum = match (minimum, first) {
                    (Some(min), Some(first)) => Some(cmp::min(min, first)),
                    (None, Some(first)) => Some(first),
                    (min, _) => min,
                };
            }

            match minimum {
                Some(min) => {
                    if min == first {
                        self.base = &self.base[1..];
                    } else {
                        let off = self.base.iter().take_while(|&x| (self.f)(x) < min).count();
                        extend(output, &self.base[..off])?;

                        self.base = &self.base[off..];
                    }
                },
                None => {
                    extend(output, self.base)?;
                    break;
                },
            }
        }
        Ok(())
    }
    
    fn iter(&'a self) -> DifferenceByKeyIter<'a, T, U, F, G, K>
    {
        DifferenceByKeyIter {
            base: self.base,
            others: self.others.clone(),
            f: &self.f,
            g: &self.g,
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
    //slices: Vec<&'a [T]>,
    base: &'a [T],
    others: Vec<&'a [U]>,
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
            if self.base.is_empty() {
                return None;
            }
            let first_base = (self.f)(&self.base[0]);
            let mut minimum = None;
            for slice in self.others.iter_mut() {
                *slice = exponential_offset_ge_by_key(slice, &first_base, &self.g);
                if !slice.is_empty() {
                    let first_other = (self.g)(&slice[0]);
                    match minimum {
                        Some(min) => {
                            minimum = Some(cmp::min(min, first_other))
                        },
                        None => {
                            minimum = Some(first_other)
                        }
                    }
                }
            }

            match minimum {
                Some(min) if min == first_base => {
                    self.base = &self.base[1..];
                },
                _ => {
                    let result = &self.base[0];
                    self.base = &self.base[1..];
                    return Some(result);
                },
            }
        }
    }
}

// This version of IntoIterator moves the contents of self into the iterator.
// Therefore the iterator owns the functions (f/g).
// The separate structs are required to not break the public API which takes the functions
//   by value instead of by reference, and doesn't require them to implement Copy/Clone.
impl<'a, T, U, F, G, K> IntoIterator for DifferenceByKey<'a, T, U, F, G, K>
where F: Fn(&T) -> K,
      G: Fn(&U) -> K,
      K: Ord,
{
    type Item = &'a T;
    type IntoIter = DifferenceByKeyIterOwning<'a, T, U, F, G, K>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            base: self.base,
            others: self.others,
            f: self.f,
            g: self.g,
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
    base: &'a [T],
    others: Vec<&'a [U]>,
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
            if self.base.is_empty() {
                return None;
            }
            let first_base = (self.f)(&self.base[0]);
            let mut minimum = None;
            for slice in self.others.iter_mut() {
                *slice = exponential_offset_ge_by_key(slice, &first_base, &self.g);
                if !slice.is_empty() {
                    let first_other = (self.g)(&slice[0]);
                    match minimum {
                        Some(min) => {
                            minimum = Some(cmp::min(min, first_other))
                        },
                        None => {
                            minimum = Some(first_other)
                        }
                    }
                }
            }

            match minimum {
                Some(min) if min == first_base => {
                    self.base = &self.base[1..];
                },
                _ => {
                    let result = &self.base[0];
                    self.base = &self.base[1..];
                    return Some(result);
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod set_to_set {
        use super::super::*;
        use crate::set::{sort_dedup_vec, SetBuf};

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Foo {
            a: i32,
            b: i8,
        }

        impl Foo {
            fn new(a: i32) -> Foo {
                Foo { a, b: 0 }
            }
        }

        #[test]
        fn one_empty_slice() {
            let a: &[Foo] = &[];

            let op = DifferenceByKey { base: a, others: vec![], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            assert_eq!(&res[..], &[]);
        }

        #[test]
        fn one_slice() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];

            let op = DifferenceByKey { base: a, others: vec![], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            assert_eq!(&res[..], &[Foo::new(1), Foo::new(2), Foo::new(3)]);
        }

        #[test]
        fn two_slices() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];
            let b = &[2, 4];

            let op = DifferenceByKey { base: a, others: vec![b], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            assert_eq!(&res[..], &[Foo::new(1), Foo::new(3)]);
        }

        #[test]
        fn two_slices_special_case() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];
            let b = &[3];

            let op = DifferenceByKey { base: a, others: vec![b], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            assert_eq!(&res[..], &[Foo::new(1), Foo::new(2)]);
        }

        #[test]
        fn three_slices() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3), Foo::new(6), Foo::new(7)];
            let b = &[2, 3, 4];
            let c = &[3, 4, 5, 7];

            let op = DifferenceByKey { base: a, others: vec![b, c], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            assert_eq!(&res[..], &[Foo::new(1), Foo::new(6)]);
        }

        quickcheck! {
            fn qc_difference(base: Vec<i32>, xss: Vec<Vec<i64>>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut base = base;
                let mut xss = xss;

                sort_dedup_vec(&mut base);

                for xs in &mut xss {
                    sort_dedup_vec(xs);
                }

                let x: SetBuf<i32> = {
                    let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                    DifferenceByKey { base: &base, others: xss, f: |&x| x, g: |&x| x as i32 }.into_set_buf()
                };

                let mut y = BTreeSet::from_iter(base);

                for v in xss {
                    let x = BTreeSet::from_iter(v.into_iter().map(|x| x as i32));
                    y = y.difference(&x).cloned().collect();
                }
                let y: Vec<_> = y.into_iter().collect();

                x.as_slice() == y.as_slice()
            }
        }
    }

    mod set_to_iter {
        use super::super::*;
        use crate::set::sort_dedup_vec;

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Foo {
            a: i32,
            b: i8,
        }

        impl Foo {
            fn new(a: i32) -> Foo {
                Foo { a, b: 0 }
            }
        }

        #[test]
        fn one_empty_slice() {
            let a: &[Foo] = &[];

            let difference = DifferenceByKey { base: a, others: vec![], f: |x| x.a, g: |&x| x };
            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(&diff_ref[..], &[]);
            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(&diff_own[..], &[]);
        }

        #[test]
        fn one_slice() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];

            let difference = DifferenceByKey { base: a, others: vec![], f: |x| x.a, g: |&x| x };
            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(&diff_ref[..], &[Foo::new(1), Foo::new(2), Foo::new(3)]);
            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(&diff_own[..], &[Foo::new(1), Foo::new(2), Foo::new(3)]);
        }

        #[test]
        fn two_slices() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];
            let b = &[2, 4];

            let difference = DifferenceByKey { base: a, others: vec![b], f: |x| x.a, g: |&x| x };
            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(&diff_ref[..], &[Foo::new(1), Foo::new(3)]);
            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(&diff_own[..], &[Foo::new(1), Foo::new(3)]);
        }

        #[test]
        fn two_slices_special_case() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3)];
            let b = &[3];

            let difference = DifferenceByKey { base: a, others: vec![b], f: |x| x.a, g: |&x| x };
            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(&diff_ref[..], &[Foo::new(1), Foo::new(2)]);
            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(&diff_own[..], &[Foo::new(1), Foo::new(2)]);
        }

        #[test]
        fn three_slices() {
            let a = &[Foo::new(1), Foo::new(2), Foo::new(3), Foo::new(6), Foo::new(7)];
            let b = &[2, 3, 4];
            let c = &[3, 4, 5, 7];

            let difference = DifferenceByKey { base: a, others: vec![b, c], f: |x| x.a, g: |&x| x };
            let diff_ref: Vec<Foo> = difference.iter().cloned().collect();
            assert_eq!(&diff_ref[..], &[Foo::new(1), Foo::new(6)]);
            let diff_own: Vec<Foo> = difference.into_iter().cloned().collect();
            assert_eq!(&diff_own[..], &[Foo::new(1), Foo::new(6)]);
        }

        quickcheck! {
            fn qc_difference(base: Vec<i32>, xss: Vec<Vec<i64>>) -> bool {
                use std::collections::BTreeSet;
                use std::iter::FromIterator;

                let mut base = base;
                let mut xss = xss;

                sort_dedup_vec(&mut base);

                for xs in &mut xss {
                    sort_dedup_vec(xs);
                }

                let x: Vec<i32> = {
                    let xss = xss.iter().map(|xs| xs.as_slice()).collect();
                    DifferenceByKey { base: &base, others: xss, f: |&x| x, g: |&x| x as i32 }.into_iter().cloned().collect()
                };

                let mut y = BTreeSet::from_iter(base);

                for v in xss {
                    let x = BTreeSet::from_iter(v.into_iter().map(|x| x as i32));
                    y = y.difference(&x).cloned().collect();
                }
                let y: Vec<_> = y.into_iter().collect();

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

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn two_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (100..200).collect();

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn three_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (1..101).collect();
        let c: Vec<_> = (2..102).collect();

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b, &c], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn three_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (34..134).collect();
        let c: Vec<_> = (66..167).collect();

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b, &c], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }

    #[bench]
    fn three_slices_big3(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).map(Foo::new).collect();
        let b: Vec<_> = (100..200).collect();
        let c: Vec<_> = (200..300).collect();

        bench.iter(|| {
            let op = DifferenceByKey { base: &a, others: vec![&b, &c], f: |x| x.a, g: |&x| x };
            let res: SetBuf<Foo> = op.into_set_buf();
            test::black_box(|| res);
        });
    }
}
