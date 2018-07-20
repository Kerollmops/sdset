pub struct Difference<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Difference<'a, T> {
    pub fn new(slices: Vec<&'a [T]>) -> Self {
        Difference { slices }
    }
}

/// Returns the slice but with its start advanced to an element
/// that is greater to the one given in parameter.
#[inline]
fn offset_gt<'a, T: 'a + PartialOrd>(slice: &'a [T], elem: &'a T) -> &'a [T] {
    match slice.iter().position(|x| x > elem) {
        Some(pos) => &slice[pos..],
        None => &[],
    }
}

impl<'a, T: Ord + Clone> Difference<'a, T> {
    pub fn into_vec(mut self) -> Vec<T> {
        let (base, others) = match self.slices.split_first_mut() {
            Some(split) => split,
            None => return Vec::new(),
        };

        let mut output = Vec::new();

        while !base.is_empty() {
            match others.iter().filter_map(|v| v.first()).min() {
                Some(min) => {
                    let len = output.len();
                    output.extend(base.iter().take_while(|&x| x != min).cloned());
                    let add = output.len() - len;

                    *base = if add < base.len() { &base[add + 1..] } else { &[] };

                    // @Improvement: advance each slice to something different
                    for slice in others.iter_mut() {
                        *slice = offset_gt(slice, min);
                    }
                },
                None => {
                    output.extend(base.iter().cloned());
                    break;
                },
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
        let union_: Vec<i32> = Difference::new(vec![]).into_vec();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_empty_slice() {
        let a: &[i32] = &[];

        let intersection_ = Difference::new(vec![a]).into_vec();
        assert_eq!(&intersection_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let union_ = Difference::new(vec![a]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 4];

        let union_ = Difference::new(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 3]);
    }

    #[test]
    fn two_slices_special_case() {
        let a = &[1, 2, 3];
        let b = &[3];

        let union_ = Difference::new(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3, 6];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let union_ = Difference::new(vec![a, b, c]).into_vec();
        assert_eq!(&union_[..], &[1, 6]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_ = Difference::new(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn bench_two_slices_big2(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (51..151).collect();

        bench.iter(|| {
            let union_ = Difference::new(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }
}
