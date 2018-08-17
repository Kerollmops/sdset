use self::Minimums::*;

pub enum Minimums<T> {
    Nothing,
    One(T),
    Two(T, T),
}

/// Returns the first values of two slices along with the indexes
/// which are the minimums (could be equal).
#[inline]
pub fn two_minimums<'a, T: 'a + Ord>(slices: &[&'a [T]]) -> Minimums<(usize, &'a T)>
{
    let mut minimums: Minimums<(_, &T)> = Nothing;

    for (index, slice) in slices.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        let current = (index, &slice[0]);
        let (_, min) = current;

        minimums = match minimums {
            One(f) | Two(f, _) if min <  f.1 => Two(current, f),
            One(f)             if min >= f.1 => Two(f, current),
            Two(f, s)          if min <  s.1 => Two(f, current),
            Nothing                          => One(current),
            other                            => other,
        };
    }

    minimums
}
