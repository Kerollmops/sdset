# sdset

Set theory applied on sorted and deduplicated slices. Much performances! Such Wow!

`sdset` stands for `sorted-deduplicated-slices-set-operations` which is a little bit too long.

[Documentation](https://docs.rs/sdset) can be found on docs.rs.

## Performances

Note about the tests, which are done on ranges of integer, if it ends with:
  - `two_slices_big`, the first slice contains `0..100` and the second has `1..101`
  - `two_slices_big2`, the first contains `0..100`, the second has `51..151`
  - `two_slices_big3`, the first contains `0..100` and the second has `100..200`
  - `three_slices_big`, the first contains `0..100`,the second has `1..101` and the third has `2..102`
  - `three_slices_big2`, the first contains `0..100`, the second has `34..134` and the third has `67..167`
  - `three_slices_big3`, the first contains `0..100`, the second has `100..200` and the third has `200..300`

These slices of ranges of integer are useful when they overlap, we can see how performances changes on different overlap slices parts.

To run the benchmarks you must enable the `unstable` feature.

```bash
$ cargo bench --features unstable
```

### BTreeSet

Here is the performances of the `BtreeSet`.

```
bench::difference_btree_two_slices_big           796 ns/iter (+/- 40)
bench::difference_btree_two_slices_big2        1,156 ns/iter (+/- 150)
bench::difference_btree_two_slices_big3        1,215 ns/iter (+/- 267)

bench::intersection_btree_two_slices_big       1,430 ns/iter (+/- 323)
bench::intersection_btree_two_slices_big2      1,130 ns/iter (+/- 335)
bench::intersection_btree_two_slices_big3        509 ns/iter (+/- 24)

bench::union_btree_two_slices_big              1,068 ns/iter (+/- 73)
bench::union_btree_two_slices_big2             1,322 ns/iter (+/- 224)
bench::union_btree_two_slices_big3             1,507 ns/iter (+/- 47)
```

```
bench::difference_btree_three_slices_big         913 ns/iter (+/- 62)
bench::difference_btree_three_slices_big2      3,016 ns/iter (+/- 235)
bench::difference_btree_three_slices_big3      7,149 ns/iter (+/- 1,815)

bench::intersection_btree_three_slices_big     7,522 ns/iter (+/- 1,057)
bench::intersection_btree_three_slices_big2    4,885 ns/iter (+/- 340)
bench::intersection_btree_three_slices_big3      626 ns/iter (+/- 83)

bench::union_btree_three_slices_big            7,270 ns/iter (+/- 389)
bench::union_btree_three_slices_big2           9,955 ns/iter (+/- 769)
bench::union_btree_three_slices_big3          15,716 ns/iter (+/- 842)
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
bench::union_sort_dedup_two_slices_big         2,393 ns/iter (+/- 285)
bench::union_sort_dedup_two_slices_big2        2,076 ns/iter (+/- 132)
bench::union_sort_dedup_two_slices_big3          824 ns/iter (+/- 154)

bench::union_sort_dedup_three_slices_big       3,545 ns/iter (+/- 288)
bench::union_sort_dedup_three_slices_big2      3,951 ns/iter (+/- 745)
bench::union_sort_dedup_three_slices_big3      1,200 ns/iter (+/- 114)
```

### sdset

And now the performances of the `sdset` library.

First with the `multi` module types, which can execute set operations on any number of slices.

Note that the rare operations that are not worth are the `union` for the `two_slices_big` test, and the `difference` for the `three_slices_big` which are slower than the `BTreeSet`one.

```
multi::difference::bench::two_slices_big         824 ns/iter (+/- 231)
multi::difference::bench::two_slices_big2        486 ns/iter (+/- 45)
multi::difference::bench::two_slices_big3        111 ns/iter (+/- 22)

multi::intersection::bench::two_slices_big     1,048 ns/iter (+/- 454)
multi::intersection::bench::two_slices_big2      694 ns/iter (+/- 20)
multi::intersection::bench::two_slices_big3      103 ns/iter (+/- 23)

multi::union::bench::two_slices_big            1,260 ns/iter (+/- 82)
multi::union::bench::two_slices_big2             730 ns/iter (+/- 124)
multi::union::bench::two_slices_big3             193 ns/iter (+/- 7)
```

```
multi::difference::bench::three_slices_big     1,219 ns/iter (+/- 225)
multi::difference::bench::three_slices_big2      825 ns/iter (+/- 58)
multi::difference::bench::three_slices_big3      112 ns/iter (+/- 6)

multi::intersection::bench::three_slices_big   1,251 ns/iter (+/- 270)
multi::intersection::bench::three_slices_big2    711 ns/iter (+/- 152)
multi::intersection::bench::three_slices_big3    104 ns/iter (+/- 3)

multi::union::bench::three_slices_big          1,613 ns/iter (+/- 80)
multi::union::bench::three_slices_big2         1,556 ns/iter (+/- 142)
multi::union::bench::three_slices_big3           304 ns/iter (+/- 209)
```

And with the `duo` modules types, where only set operations on two slices are possible.

```
duo::difference::bench::two_slices_big           495 ns/iter (+/- 27)
duo::difference::bench::two_slices_big2          285 ns/iter (+/- 14)
duo::difference::bench::two_slices_big3           76 ns/iter (+/- 21)

duo::intersection::bench::two_slices_big         117 ns/iter (+/- 36)
duo::intersection::bench::two_slices_big2        116 ns/iter (+/- 7)
duo::intersection::bench::two_slices_big3         74 ns/iter (+/- 15)

duo::union::bench::union_two_slices_big          187 ns/iter (+/- 31)
duo::union::bench::union_two_slices_big2         178 ns/iter (+/- 49)
duo::union::bench::union_two_slices_big3         135 ns/iter (+/- 3)
```
