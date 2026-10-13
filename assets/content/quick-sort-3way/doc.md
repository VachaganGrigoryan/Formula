# Quick sort (3-way)

## Overview

Three-way quick sort splits the list around a pivot into **three** bands instead of two: values smaller than the pivot, values equal to it, and values larger. Every copy of the pivot value reaches its final place in the same pass, and the recursive calls see only the strictly smaller and strictly larger values.

That matters because real data is full of duplicates: categories, ratings, status flags, country codes. Ordinary 2-way quick sort keeps re-partitioning equal values, and on a list of identical values it degrades to quadratic time. The 3-way version finishes such a list in one linear pass, and on any list its cost tracks how many *distinct* values there are. On all-distinct data it behaves like ordinary quick sort.

The partition is Edsger Dijkstra's solution to the **Dutch national flag** problem: rearrange red, white and blue items into three bands in one pass. The app uses the first element `a[lo]` as the pivot.

## Formula

```math
pivot v = a[lo]
loop invariant:  a[lo..lt−1] < v,  a[lt..i−1] = v,  a[i..gt] unseen,  a[gt+1..hi] > v
after partition:  a[lo..lt−1] < v = a[lt..gt] < a[gt+1..hi]
H = − Σ_{k} p_{k} log_{2} p_{k}
C(n) ≈ (2 ln 2) · n · H ≈ 1.39 · n · H
```

| Symbol | Meaning |
| --- | --- |
| `v` | pivot value, copied from `a[lo]` before the loop |
| `lt` | first slot of the equal band (the pivot shown in pink sits here) |
| `i` | next unseen value |
| `gt` | last unseen value; everything after it is larger than `v` |
| `p_k` | fraction of the values equal to the k-th distinct key |
| `H` | Shannon entropy of the key frequencies, in bits |
| `C(n)` | average number of three-way comparisons for n values |

With all keys distinct, H = log₂ n and the formula gives the familiar 1.39 n log₂ n of quick sort. With d equally common values, H = log₂ d, a constant: the sort runs in linear time. Since no comparison sort can beat about n·H − n comparisons, 3-way quick sort is entropy-optimal up to a constant factor.

## Intuition

The slice is sewn into a flag from both ends. Each step looks at `a[i]`, classifies it once, and shrinks the unseen region by exactly one slot:

- **smaller**: swap it with `a[lt]`, the first pivot copy. The small band grows by one, the equal band shifts one slot right; advance `lt` and `i`.
- **larger**: swap it with `a[gt]` and decrease `gt`. Do *not* advance `i`: the value that just arrived from `gt` has not been looked at.
- **equal**: leave it; advance `i`, and the equal band grows.

Why duplicates stop costing anything: each partition finalizes *all* copies of one value, and the recursive calls never see that value again. With d distinct values the recursion can be at most d levels deep, and each level costs at most n comparisons, so the total is at most about d·n. A list of identical values takes n − 1 comparisons and no swaps.

## Algorithm

1. If the slice has fewer than two values, it is sorted; stop.
2. Copy the pivot `v = a[lo]`. Set `lt = lo`, `i = lo + 1`, `gt = hi`.
3. While `i <= gt`, compare `a[i]` with `v` and apply one of the three cases above.
4. Now `a[lt..gt]` all equal `v` and are in their final places.
5. Sort `a[lo..lt-1]` and `a[gt+1..hi]` recursively.

```
quick_sort_3way(a, lo, hi):
    if lo >= hi: return
    v = a[lo]; lt = lo; i = lo + 1; gt = hi
    while i <= gt:
        if a[i] < v:   swap a[lt], a[i]; lt += 1; i += 1
        elif a[i] > v: swap a[i], a[gt]; gt -= 1
        else:          i += 1
    quick_sort_3way(a, lo, lt - 1)
    quick_sort_3way(a, gt + 1, hi)
```

## Complexity

| Case | Time | Notes |
| --- | --- | --- |
| Best | O(n) | all values equal, or a constant number of distinct values |
| Average | O(n log n) | O(n·H) in general; 1.39 n log₂ n compares on distinct keys |
| Worst | O(n²) | distinct keys already sorted or reversed, with a first-element pivot |
| Space | O(log n) average | recursion stack; O(n) worst case |
| Stable | No | swaps across the slice reorder equal values |
| In place | Yes | only swaps inside the array |

Comparisons: exactly hi − lo three-way comparisons per partition, one per value looked at. Swaps: at most one for every value that is not equal to the pivot, so on distinct keys almost every comparison is followed by a swap, several times more swaps than Hoare's 2-way partition. On data with few unique values both counts collapse.

## Parameters in the app

- **Data**: choose **Few unique** (four distinct values) and compare the step and comparison counts with the Quick sort page on the same list. Random, Nearly sorted and Reversed give all-distinct values; the last two show the quadratic worst case of a first-element pivot.
- **List size** (5 to 200) and **Randomize list**: the input, shared by every sorting page.
- **Draw as** (Bars, Radial, Color spectrum, Dot scatter) and **Sound** (a tone per value touched).
- **Speed**, **Step**, and the Restart, Step back, Play / Pause, Step and Finish buttons (or Space and the arrow keys) replay the recorded steps.
- Colors: red marks `a[i]` and the pivot being compared, amber a swap, pink the pivot at `lt`, green final values. When a partition ends, its whole equal band turns green. Bars outside the current call are faded, and the bracket rows show the open recursive calls.

## Numerical notes and pitfalls

- **Advancing i after a swap with gt** is the classic bug: the incoming value is skipped and may end up on the wrong side.
- **Read the pivot once.** Keep `v` in a variable; `a[lo]` is overwritten by the first swap.
- **One comparison, three outcomes.** Written as `<` then `>`, a step can cost two binary comparisons; a single `cmp` (Rust `Ordering`, a C comparator returning −1, 0 or 1) avoids that.
- **NaN** is neither smaller, larger nor equal, so the final `else` branch silently files it with the pivot. Remove NaNs or use a total order first.
- **Pivot choice**: a first-element pivot is quadratic on sorted input. Pick a random element or the median of three and swap it to `a[lo]` before partitioning.
- **Many swaps on distinct keys**: if duplicates are rare, a 2-way Hoare partition is faster; Bentley and McIlroy's variant keeps equal values at the two ends and swaps them into the middle only at the end.

## Applications

- Sorting keys with many repeats: categories, enum values, ratings from 1 to 5, dates at day resolution.
- Library sorts: Bentley and McIlroy's three-way `qsort` in C libraries; pattern-defeating quicksort (pdqsort, in Rust and Go) gets the same effect by detecting a pivot equal to the previous one.
- One-pass three-class partitioning on its own, such as sorting an array of 0s, 1s and 2s, or splitting around a value in quickselect.
- Multikey quicksort (three-way radix quicksort) for strings, which partitions on one character at a time.

## History

Dijkstra posed and solved the Dutch national flag problem in A Discipline of Programming (1976) as an exercise in deriving programs from invariants; the three bands are the red, white and blue stripes of the Dutch flag. Robert Sedgewick analyzed quick sort with equal keys in 1977. Jon Bentley and Douglas McIlroy made three-way partitioning practical in "Engineering a Sort Function" (1993), and Bentley and Sedgewick extended it to strings in 1997. Sedgewick and Bentley's 2002 talk "Quicksort is Optimal" showed that three-way quick sort is entropy-optimal.

## References

- E. W. Dijkstra, A Discipline of Programming, Prentice-Hall, 1976.
- J. L. Bentley and M. D. McIlroy, "Engineering a Sort Function", Software: Practice and Experience 23(11), 1993.
- R. Sedgewick, "Quicksort with Equal Keys", SIAM Journal on Computing 6(2), 1977.
- R. Sedgewick and K. Wayne, Algorithms, 4th edition, section 2.3 (three-way partitioning).
- J. L. Bentley and R. Sedgewick, "Fast Algorithms for Sorting and Searching Strings", SODA 1997.
- Wikipedia, "Dutch national flag problem".
