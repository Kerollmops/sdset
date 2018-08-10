# sdset

Set theory applied on sorted and deduplicated slices. Much performances! Such Wow!

`sdset` stands for `sorted-deduplicated-slices-set-operations` which is a little bit too long.

[Documentation](https://docs.rs/sdset) can be found on docs.rs.

## Performances

Note about the tests that ends with:
  - `_big`, sets are done on two ranges of integer, the first is `0..100` and the second is `1..101`
  - `_big2`, sets are done on two ranges of integer, the first is `0..100` and the second is `51..151`
  - `_big3`, sets are done on two ranges of integer, the first is `0..100` and the second is `100..200`

To run the benchmarks you must enable the `unstable` feature.

```bash
$ cargo bench --features unstable
```

### BTreeSet

These slices of ranges of integer are useful when they overlap, we can see how performances changes on different overlap slice parts.

Here is the performances of the `BtreeSet`.

```
bench::difference_btree_two_slices_big      ... bench:         781 ns/iter (+/- 150)
bench::difference_btree_two_slices_big2     ... bench:       1,165 ns/iter (+/- 68)
bench::difference_btree_two_slices_big3     ... bench:       1,234 ns/iter (+/- 317)

bench::intersection_btree_two_slices_big    ... bench:       1,464 ns/iter (+/- 513)
bench::intersection_btree_two_slices_big2   ... bench:       1,142 ns/iter (+/- 521)
bench::intersection_btree_two_slices_big3   ... bench:         557 ns/iter (+/- 121)

bench::union_btree_two_slices_big           ... bench:       1,061 ns/iter (+/- 163)
bench::union_btree_two_slices_big2          ... bench:       1,298 ns/iter (+/- 448)
bench::union_btree_two_slices_big3          ... bench:       1,484 ns/iter (+/- 410)
```

### accumulate, sort and dedup

The performances of a `Vec` that is extended to contain all values then `sort` and `dedup`.
Note that works only for the _union_ set operation.

```rust
fn sort_dedup<T: Ord + Clone>(a: &[T], b: &[T]) -> Vec<T>
{
    let mut elements = vec![&a, &b].into_iter().flatten().cloned().collect();
    elements.sort_unstable();
    elements.dedup();
    elements
}
```

```
bench::union_sort_dedup_two_slices_big      ... bench:       2,374 ns/iter (+/- 433)
bench::union_sort_dedup_two_slices_big2     ... bench:       2,032 ns/iter (+/- 714)
bench::union_sort_dedup_two_slices_big3     ... bench:         835 ns/iter (+/- 37)
```

### sdset

And now the performances of the `sdset` library.

First with the `multi` module types. Note that the only operation that is not worth is the `union` for the `_big` test, which is slower than the `BTreeSet`one.

```
multi::difference::bench::two_slices_big    ... bench:         781 ns/iter (+/- 126)
multi::difference::bench::two_slices_big2   ... bench:         440 ns/iter (+/- 8)
multi::difference::bench::two_slices_big3   ... bench:         107 ns/iter (+/- 40)

multi::intersection::bench::two_slices_big  ... bench:       1,106 ns/iter (+/- 348)
multi::intersection::bench::two_slices_big2 ... bench:         702 ns/iter (+/- 18)
multi::intersection::bench::two_slices_big3 ... bench:         102 ns/iter (+/- 47)

multi::union::bench::two_slices_big         ... bench:       1,230 ns/iter (+/- 32)
multi::union::bench::two_slices_big2        ... bench:         702 ns/iter (+/- 120)
multi::union::bench::two_slices_big3        ... bench:         188 ns/iter (+/- 55)
```

And with the `duo` modules types.

```
duo::difference::bench::two_slices_big      ... bench:         496 ns/iter (+/- 42)
duo::difference::bench::two_slices_big2     ... bench:         284 ns/iter (+/- 9)
duo::difference::bench::two_slices_big3     ... bench:          77 ns/iter (+/- 4)

duo::intersection::bench::two_slices_big    ... bench:         110 ns/iter (+/- 7)
duo::intersection::bench::two_slices_big2   ... bench:         107 ns/iter (+/- 9)
duo::intersection::bench::two_slices_big3   ... bench:          70 ns/iter (+/- 7)

duo::union::bench::union_two_slices_big     ... bench:         187 ns/iter (+/- 12)
duo::union::bench::union_two_slices_big2    ... bench:         169 ns/iter (+/- 15)
duo::union::bench::union_two_slices_big3    ... bench:         135 ns/iter (+/- 23)
```
