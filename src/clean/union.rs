pub struct Union<'a, T: 'a> {
    slices: Vec<&'a [T]>,
}

impl<'a, T> Union<'a, T> {
    pub fn new(slices: Vec<&'a [T]>) -> Self {
        Union { slices }
    }
}

impl<'a, T: Ord + Clone> Union<'a, T> {
    pub fn into_vec(mut self) -> Vec<T> {
        let mut output = match self.slices.first() {
            Some(slice) => Vec::with_capacity(slice.len()),
            None => Vec::new(),
        };

        while let Some(min) = self.slices.iter().filter_map(|v| v.first()).min() {
            // save the element
            output.push(min.clone());
            // move slices forward
            for slice in self.slices.iter_mut().filter(|s| !s.is_empty()) {
                if slice[0] == *min {
                    *slice = &slice[1..];
                }
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
        let union_: Vec<i32> = Union::new(vec![]).into_vec();
        assert_eq!(&union_[..], &[]);
    }

    #[test]
    fn one_slice() {
        let a = &[1, 2, 3];

        let union_ = Union::new(vec![a]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3]);
    }

    #[test]
    fn two_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];

        let union_ = Union::new(vec![a, b]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn three_slices() {
        let a = &[1, 2, 3];
        let b = &[2, 3, 4];
        let c = &[3, 4, 5];

        let union_ = Union::new(vec![a, b, c]).into_vec();
        assert_eq!(&union_[..], &[1, 2, 3, 4, 5]);
    }

    #[bench]
    fn bench_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let union_ = Union::new(vec![&a, &b]).into_vec();
            test::black_box(|| union_);
        });
    }

    #[bench]
    fn bench_btree_two_slices_big(bench: &mut Bencher) {
        use std::collections::BTreeSet;
        use std::iter::FromIterator;

        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let mut set = BTreeSet::new();
            for slice in vec![&a, &b] {
                let slice = BTreeSet::from_iter(slice.into_iter().cloned());
                set = set.union(&slice).cloned().collect();
            }

            test::black_box(|| set);
        });
    }

    #[bench]
    fn bench_sort_dedup_two_slices_big(bench: &mut Bencher) {
        let a: Vec<_> = (0..100).collect();
        let b: Vec<_> = (1..101).collect();

        bench.iter(|| {
            let mut elements: Vec<_> = vec![&a, &b].into_iter().flatten().cloned().collect();
            elements.sort_unstable();
            elements.dedup();

            test::black_box(|| elements);
        });
    }
}
