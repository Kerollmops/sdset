# sdset

Set theory applied on sorted and deduplicated slices. Much performances! Such Wow!

`sdset` is for `sorted-deduplicated-slices-set-operation` which is a little bit too long.

[Documentation](https://docs.rs/sdset) can be found on docs.rs.

## Performances

Note about the tests that ends with:
  - `_big`, sets are done on two ranges of integer, the first is `0..100` and the second is `1..101`
  - `_big2`, sets are done on two ranges of integer, the first is `0..100` and the second is `51..151`
  - `_big3`, sets are done on two ranges of integer, the first is `0..100` and the second is `100..200`

### BTreeSet

These slices of ranges of integer are useful when they overlap, we can see how performances changes on different overlap slice parts.

Here is the performances of the `BtreeSet`.

```
tests::bench_difference_btree_two_slices_big      ... bench:         774 ns/iter (+/- 54)
tests::bench_difference_btree_two_slices_big2     ... bench:       1,137 ns/iter (+/- 151)
tests::bench_difference_btree_two_slices_big3     ... bench:       1,218 ns/iter (+/- 78)

tests::bench_intersection_btree_two_slices_big    ... bench:       1,453 ns/iter (+/- 84)
tests::bench_intersection_btree_two_slices_big2   ... bench:       1,142 ns/iter (+/- 36)
tests::bench_intersection_btree_two_slices_big3   ... bench:         553 ns/iter (+/- 112)

tests::bench_union_btree_two_slices_big           ... bench:       1,066 ns/iter (+/- 578)
tests::bench_union_btree_two_slices_big2          ... bench:       1,300 ns/iter (+/- 202)
tests::bench_union_btree_two_slices_big3          ... bench:       1,462 ns/iter (+/- 55)
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
tests::bench_union_sort_dedup_two_slices_big      ... bench:       2,307 ns/iter (+/- 461)
tests::bench_union_sort_dedup_two_slices_big2     ... bench:       2,019 ns/iter (+/- 326)
tests::bench_union_sort_dedup_two_slices_big3     ... bench:         825 ns/iter (+/- 30)
```

### sdset

And now the performances of the `sdset` library.

First with the `multi` module types. Note that the only operation that is not worth is the `union` for the `_big` test, which is slower than the `BTreeSet`one.

```
multi::difference::tests::bench_two_slices_big    ... bench:         782 ns/iter (+/- 36)
multi::difference::tests::bench_two_slices_big2   ... bench:         445 ns/iter (+/- 36)
multi::difference::tests::bench_two_slices_big3   ... bench:         103 ns/iter (+/- 17)

multi::intersection::tests::bench_two_slices_big  ... bench:       1,085 ns/iter (+/- 60)
multi::intersection::tests::bench_two_slices_big2 ... bench:         704 ns/iter (+/- 71)
multi::intersection::tests::bench_two_slices_big3 ... bench:         102 ns/iter (+/- 20)

multi::union::tests::bench_two_slices_big         ... bench:       1,165 ns/iter (+/- 57)
multi::union::tests::bench_two_slices_big2        ... bench:         663 ns/iter (+/- 65)
multi::union::tests::bench_two_slices_big3        ... bench:         177 ns/iter (+/- 53)
```

And with the `duo` modules types.

```
duo::difference::tests::bench_two_slices_big      ... bench:         474 ns/iter (+/- 31)
duo::difference::tests::bench_two_slices_big2     ... bench:         285 ns/iter (+/- 60)
duo::difference::tests::bench_two_slices_big3     ... bench:          74 ns/iter (+/- 7)

duo::intersection::tests::bench_two_slices_big    ... bench:         110 ns/iter (+/- 8)
duo::intersection::tests::bench_two_slices_big2   ... bench:         105 ns/iter (+/- 19)
duo::intersection::tests::bench_two_slices_big3   ... bench:          73 ns/iter (+/- 8)

duo::union::tests::union_bench_two_slices_big     ... bench:         183 ns/iter (+/- 59)
duo::union::tests::union_bench_two_slices_big2    ... bench:         164 ns/iter (+/- 12)
duo::union::tests::union_bench_two_slices_big3    ... bench:         142 ns/iter (+/- 6)
```
