use std::cmp::{self, Ordering};

pub trait Algorithm {
    fn search_by<T, F>(slice: &[T], f: F) -> Result<usize, usize>
    where F: FnMut(&T) -> Ordering;

    fn search_by_key<T, B, F>(slice: &[T], b: &B, mut f: F) -> Result<usize, usize>
    where F: FnMut(&T) -> B,
          B: Ord,
    {
        Self::search_by(slice, |k| f(k).cmp(b))
    }

    fn search<T: Ord>(slice: &[T], elem: &T) -> Result<usize, usize> {
        Self::search_by(slice, |x| x.cmp(elem))
    }

    fn offset_ge<'a, T: Ord>(slice: &'a [T], elem: &T) -> &'a [T] {
        Self::offset_ge_by(slice, |x| x.cmp(elem))
    }

    fn offset_ge_by_key<'a, T, B, F>(slice: &'a [T], b: &B, mut f: F) -> &'a [T]
    where F: FnMut(&T) -> B,
          B: Ord,
    {
        Self::offset_ge_by(slice, |x| f(x).cmp(b))
    }

    fn offset_ge_by<T, F>(slice: &[T], f: F) -> &[T]
    where F: FnMut(&T) -> Ordering,
    {
        match Self::search_by(slice, f) {
            Ok(pos) => &slice[pos..],
            Err(pos) => &slice[pos..],
        }
    }
}

pub enum Linear {}

impl Algorithm for Linear {
    fn search_by<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
    where F: FnMut(&T) -> Ordering,
    {
        for (i, x) in slice.iter().enumerate() {
            match f(x) {
                Ordering::Less => (),
                Ordering::Equal => return Ok(i),
                Ordering::Greater => return Err(i),
            }
        }

        return Err(slice.len())
    }
}

pub enum Binary {}

impl Algorithm for Binary {
    fn search_by<T, F>(slice: &[T], f: F) -> Result<usize, usize>
    where F: FnMut(&T) -> Ordering,
    {
        slice.binary_search_by(f)
    }
}

pub enum Exponential {}

impl Algorithm for Exponential {
    fn search_by<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
    where F: FnMut(&T) -> Ordering,
    {
        let mut index = 1;
        while index < slice.len() && f(&slice[index]) == Ordering::Less {
            index *= 2;
        }

        let half_bound = index / 2;
        let bound = cmp::min(index + 1, slice.len());

        match slice[half_bound..bound].binary_search_by(f) {
            Ok(pos) => Ok(half_bound + pos),
            Err(pos) => Err(half_bound + pos),
        }
    }
}
