#![feature(test)]

extern crate test;

// FIXME allow to use #![no_std]
use std::cmp::Ordering;

#[inline]
fn skip_duplicates<'a, T: Eq>(slice: &'a [T], elem: &'a T) -> &'a [T] {
    let count = slice.iter().take_while(|x| *x == elem).count();
    &slice[count..]
}

pub fn union_two_slices<T>(mut a: &[T], mut b: &[T]) -> Vec<T>
where T: Ord + Clone
{
    let mut output = Vec::new();

    while !a.is_empty() && !b.is_empty() {
        match a[0].cmp(&b[0]) {
            Ordering::Less => {
                output.push(a[0].clone());
                a = skip_duplicates(a, &a[0]);
            },
            Ordering::Equal => {
                output.push(a[0].clone());
                a = skip_duplicates(a, &a[0]);
                b = skip_duplicates(b, &b[0]);
            },
            Ordering::Greater => {
                output.push(b[0].clone());
                b = skip_duplicates(b, &b[0]);
            },
        }
    }

    // if b is empty before a, add all remaining elements of a
    while !a.is_empty() {
        output.push(a[0].clone());
        a = skip_duplicates(a, &a[0]);
    }

    // idem but if a is empty before b
    while !b.is_empty() {
        output.push(b[0].clone());
        b = skip_duplicates(b, &b[0]);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn union_two_slices_easy() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_duplicates() {
        let a = &[1, 2, 2, 3, 3];
        let b = &[2, 3, 3, 4];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_duplicates_at_end() {
        let a = &[1, 2, 3, 4];
        let b = &[2, 3, 4];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[1, 2, 3, 4]);
    }

    #[test]
    fn union_two_slices_second_empty() {
        let a = &[1, 2, 2, 3, 3];
        let b = &[];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[1, 2, 3]);
    }

    #[test]
    fn union_two_slices_first_empty() {
        let a = &[];
        let b = &[2, 3, 3, 4];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[2, 3, 4]);
    }

    #[test]
    fn union_two_slices_same_elem() {
        let a = &[1, 1, 1, 1];
        let b = &[1, 1, 1, 1, 1];

        let union_ = union_two_slices(a, b);

        assert_eq!(&union_, &[1]);
    }

    #[bench]
    fn bench_two_slices_easy(b: &mut Bencher) {
        b.iter(|| {
            let a = &[1, 2, 3];
            let b = &[2, 3, 4];

            let union_ = union_two_slices(a, b);
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn bench_two_slices_duplicates(b: &mut Bencher) {
        b.iter(|| {
            let a = &[1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];
            let b = &[2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4];

            let union_ = union_two_slices(a, b);
            test::black_box(|| union_);
        });
    }
}
