use std::cmp;

use self::Equality::*;

pub struct Intersection<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Intersection<'a, T> {
    pub fn new(slices: Vec<&'a [T]>) -> Self {
        Intersection { slices }
    }
}

enum Equality<'a, T: 'a> {
    NotEqual(&'a T),
    Equal(&'a T),
}

#[inline]
fn test_equality<'a, I, T: 'a + Ord>(iter: I) -> Option<Equality<'a, T>>
where I: Iterator<Item = &'a T>
{
    let mut is_equal = true;
    let mut max = None;
    for x in iter {
        if max.is_some() { is_equal = max == Some(x) }
        max = cmp::max(max, Some(x));
    }
    max.map(|max| if is_equal { Equal(max) } else { NotEqual(max) })
}

/// Returns the slice but with its start advanced to an element
/// that is equal to the one given in parameter.
#[inline]
fn offset_eq<'a, T: 'a + Eq>(slice: &'a [T], elem: &'a T) -> &'a [T] {
    match slice.iter().position(|x| x == elem) {
        Some(pos) => &slice[pos..],
        None => &[],
    }
}

impl<'a, T: Ord + Clone> Intersection<'a, T> {
    pub fn into_vec(mut self) -> Vec<T> {
        let mut output = Vec::new();

        'outer: loop {
            match test_equality(self.slices.iter().filter_map(|s| s.first())) {
                Some(Equal(x)) => {
                    output.push(x.clone());
                    for slice in &mut self.slices {
                        *slice = &slice[1..];
                        if slice.is_empty() { break 'outer }
                    }
                },
                Some(NotEqual(x)) => {
                    for slice in &mut self.slices {
                        *slice = offset_eq(slice, x);
                        if slice.is_empty() { break 'outer }
                    }
                },
                None => break,
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::{self, Bencher};

    #[test]
    fn no_slice() {
        let intersection_: Vec<i32> = Intersection::new(vec![]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let intersection_ = Intersection::new(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let intersection_ = Intersection::new(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let intersection_ = Intersection::new(vec![a, b]).into_vec();
        assert_eq!(&intersection_[..], &[2, 3]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let intersection_ = Intersection::new(vec![a, b, c]).into_vec();
        assert_eq!(&intersection_[..], &[3]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let intersection_ = Intersection::new(vec![&a, &b]).into_vec();
            test::black_box(|| intersection_);
        });
    }
}
