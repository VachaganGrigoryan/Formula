# Quick sort

## Overview

Quick sort is a divide-and-conquer sort. It picks one value, the **pivot**, and *partitions* the list so that everything not larger than the pivot sits on its left and everything larger on its right. The pivot is then in its final place, and the two sides are sorted the same way until every piece has at most one value.

The work happens before the recursive calls (merge sort does it after them), in place, by swapping. On random data it is usually the fastest comparison sort, which is why most standard libraries build their unstable sort on it. Its weakness is the pivot: with a bad one, every partition peels off a single value and the running time becomes quadratic.

The app uses the **Lomuto partition** with the **last element as pivot**, the version taught in most textbooks.

## Formula

```math
pivot v = a[hi]
after partition:  a[lo..p−1] ≤ v = a[p] < a[p+1..hi]
T(n) = T(k) + T(n − 1 − k) + (n − 1)
C_{avg}(n) = 2(n + 1) H_{n} − 4n ≈ 1.39 · n · log_{2} n
C_{worst}(n) = n(n − 1) / 2
```

| Symbol | Meaning |
| --- | --- |
| `a[lo..hi]` | the slice being partitioned (inclusive bounds) |
| `v` | pivot value, taken from the last slot `a[hi]` |
| `p` | the pivot's final index, returned by the partition |
| `k` | number of values that end up left of the pivot |
| `T(n)`, `C(n)` | comparisons to sort n values |
| `H_n` | harmonic number 1 + 1/2 + … + 1/n ≈ ln n |

Each partition of a slice of length n costs exactly n − 1 comparisons. The average assumes all orderings are equally likely, so the pivot's rank is uniform; the worst case is a pivot that is always the smallest or largest value.

## Intuition

A partition is a sweep with two fingers. `j` walks across the slice; `i` marks the end of a growing "small" region at the front. Whenever `a[j] <= v`, that value is swapped into the small region and `i` advances. When `j` reaches the pivot, everything in `a[lo..i-1]` is at most `v` and everything in `a[i..hi-1]` is larger, so swapping the pivot into slot `i` puts it exactly between the two groups. Picture a horizontal line at the pivot's height: during the sweep, bars below it drift to the front, bars above it stay behind.

Why n log n on average: each level of recursion costs about n comparisons in total, and a random pivot lands in the middle half of its slice half the time, which keeps the depth near log n. An exact view: values of ranks i < j are compared only if one of them is the first pivot chosen among the j − i + 1 values between them, which has probability 2 / (j − i + 1). Summing over all pairs gives the 2n ln n average.

## Algorithm

1. If the slice has fewer than two values, it is sorted; stop.
2. Take the last value `a[hi]` as pivot `v`. Set `i = lo`.
3. For `j` from `lo` to `hi - 1`: if `a[j] <= v`, swap `a[i]` and `a[j]`, then increase `i`.
4. Swap `a[i]` with `a[hi]`. The pivot is now at `p = i`, in its final place.
5. Sort `a[lo..p-1]` and `a[p+1..hi]` recursively.

```
quick_sort(a, lo, hi):
    if lo < hi:
        p = partition(a, lo, hi)
        quick_sort(a, lo, p - 1)
        quick_sort(a, p + 1, hi)

partition(a, lo, hi):
    v = a[hi]; i = lo
    for j in lo .. hi - 1:
        if a[j] <= v: swap a[i], a[j]; i += 1
    swap a[i], a[hi]
    return i
```

## Complexity

| Case | Time | Notes |
| --- | --- | --- |
| Best | O(n log n) | every pivot lands near the middle |
| Average | O(n log n) | about 1.39 n log₂ n comparisons |
| Worst | O(n²) | already sorted or reversed input with this last-element pivot |
| Space | O(log n) average | the recursion stack; O(n) in the worst case |
| Stable | No | long-distance swaps reorder equal values |
| In place | Yes | only swaps inside the array |

Comparisons: exactly n − 1 per partition of n values, n(n − 1)/2 in the worst case. Swaps: Lomuto does roughly one swap per value that is not larger than the pivot, about n ln n on average, which is around three times as many as Hoare's original two-pointer partition. On sorted input it compares n(n − 1)/2 times but swaps nothing, since every value is already in the small region.

## Parameters in the app

- **List size** (5 to 200) and **Data** (Random, Nearly sorted, Reversed, Few unique): the input, shared by every sorting page. Nearly sorted and Reversed show the quadratic worst case of a last-element pivot. **Randomize list** draws a new arrangement.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter. **Sound** plays a tone whose pitch follows the value being touched.
- **Speed** and **Step**: replay rate in steps per second, and a scrubber over the recorded steps; Restart, Step back, Play / Pause, Step and Finish, or Space and the arrow keys.
- Colors: blue unsorted, red comparing, amber swapping, pink the pivot, green in final place. Bars outside the current partition are faded; the bracket rows under the bars show the open recursive calls, one row per depth.
- The status line counts steps, comparisons, swaps and the recursion depth.

## Numerical notes and pitfalls

- **Sorted input is the worst case** for a first- or last-element pivot. Use median-of-three (first, middle, last) or a random pivot; libraries also fall back to heap sort when recursion gets too deep (introsort).
- **Many equal values** hurt Lomuto badly: if every value equals the pivot, every value goes left and each partition removes only one value. Use a three-way partition (see Quick sort (3-way)) or Hoare's scheme, which splits equal values evenly.
- **Stack depth.** Recurse into the smaller side and loop on the larger one; that bounds the stack at O(log n) even in the worst case.
- **Off-by-one errors**: mixing inclusive and exclusive bounds, or forgetting the final pivot swap, gives infinite recursion or unsorted output. With unsigned indices, `p - 1` underflows when `p = 0`.
- **Small slices**: below roughly 10 to 20 values, real implementations switch to insertion sort.
- **Floating-point keys**: a NaN breaks the total order that partitioning relies on. Filter NaNs or use a total ordering.

## Applications

- Standard library unstable sorts: introsort in C++ `std::sort`, pattern-defeating quicksort (pdqsort) and its descendants in Go and in Rust's `sort_unstable`, dual-pivot quick sort for primitive arrays in Java.
- Quickselect: the same partition finds the k-th smallest value (medians, percentiles) in linear expected time.
- In-memory sorting in databases and analytics engines when stability is not needed.
- Partitioning itself: moving items that match a predicate to the front of an array.

## History

Tony Hoare invented quick sort in 1959 as a visiting student at Moscow State University, working on machine translation, where he needed to sort the words of a sentence before looking them up. He published it as Algorithm 64 in Communications of the ACM (1961) and analyzed it in "Quicksort" in The Computer Journal (1962); the name is his. Hoare's partition used two pointers moving towards each other. The simpler single-pointer scheme shown here is credited to Nico Lomuto and was popularized by Jon Bentley's Programming Pearls and by Cormen, Leiserson, Rivest and Stein's textbook. Robert Sedgewick's 1975 PhD thesis gave the definitive analysis, and David Musser's introsort (1997) removed the quadratic worst case in practice.

## References

- C. A. R. Hoare, "Quicksort", The Computer Journal 5(1), 1962.
- T. H. Cormen, C. E. Leiserson, R. L. Rivest, C. Stein, Introduction to Algorithms, chapter 7 "Quicksort".
- R. Sedgewick and K. Wayne, Algorithms, 4th edition, section 2.3.
- J. Bentley, Programming Pearls, 2nd edition, column 11 "Sorting".
- D. E. Knuth, The Art of Computer Programming, Volume 3: Sorting and Searching, section 5.2.2.
- Wikipedia, "Quicksort".
