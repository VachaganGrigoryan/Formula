# Merge sort

## Overview

Merge sort is the classic **divide and conquer** sort. It splits the list into two halves, sorts each half recursively, and then **merges** the two sorted halves into one sorted list by repeatedly taking the smaller of their two front values. A list of one value is already sorted, so the recursion bottoms out immediately.

It runs in O(n log n) time on every input, it is stable, and its merging step reads and writes data strictly in order, which makes it the method of choice for linked lists and for data too large to fit in memory. The price is an extra buffer as large as the input.

## Formula

```math
T(n) = T(floor(n/2)) + T(ceil(n/2)) + Θ(n),   T(1) = Θ(1)
T(n) = Θ(n log₂ n)
C_{worst}(n) = n · ceil(log₂ n) − 2^{ceil(log₂ n)} + 1
log₂(n!) ≈ n log₂ n − 1.44 n
```

| Symbol | Meaning |
| --- | --- |
| `n` | number of values |
| `T(n)` | running time on n values |
| `Θ(n)` | the merge step: linear in the length of the range |
| `C_worst(n)` | most comparisons the top-down version can make |
| `log₂(n!)` | the minimum worst-case comparisons of any comparison sort |

The recursion has about log₂ n levels and each level merges a total of n values, which gives n log₂ n. The worst-case comparison count is within about n/2 comparisons of the information-theoretic lower bound in the last line, so merge sort is close to optimal in comparisons.

## Intuition

Merging is the easy part. Given two sorted piles face up, the smallest card overall is on top of one of them; take it, and repeat. Every comparison places one value, so merging piles of sizes p and q needs at most p + q − 1 comparisons and exactly p + q writes.

Sorting then reduces to merging: split until every piece has one value, and merge pieces pairwise back up. In the app the rows under the bars show this recursion. Each open call is one row, outermost on top; the bright row is the merge in progress with its left and right halves in two colors, and the bars outside the current range fade. The top-level merge is the last one, and as it writes each value that value is in its final place.

## Algorithm

1. If the range `lo..hi` has fewer than two values, return.
2. Let `mid = lo + (hi − lo) / 2`. Sort `lo..mid` and `mid..hi` recursively.
3. Copy the two halves aside as `L` and `R`.
4. Walk both copies with indices `i` and `j`. Write the smaller front value into `a[k]` and advance its index; on a tie take from `L`.
5. When one copy runs out, copy the rest of the other.

```
merge_sort(a, lo, hi):            # sorts a[lo..hi-1]
    if hi - lo < 2: return
    mid = lo + (hi - lo) / 2
    merge_sort(a, lo, mid)
    merge_sort(a, mid, hi)
    L = a[lo..mid-1];  R = a[mid..hi-1]
    i = j = 0
    for k in lo .. hi-1:
        if j == len(R) or (i < len(L) and L[i] <= R[j]):
            a[k] = L[i];  i = i + 1
        else:
            a[k] = R[j];  j = j + 1
```

## Complexity

| Measure | Value |
| --- | --- |
| Best time | O(n log n): about (n/2) log₂ n comparisons on sorted input |
| Average time | O(n log n): about n log₂ n − 1.26 n comparisons |
| Worst time | O(n log n): at most n · ceil(log₂ n) − 2^ceil(log₂ n) + 1 comparisons |
| Writes | n per level of recursion, n · ceil(log₂ n) at most |
| Extra space | O(n) buffer plus O(log n) recursion stack |
| Stable | yes, if ties take from the left half |
| In place | no (in-place merging exists but is complicated and slower) |
| Adaptive | not in this form; natural merge sort and Timsort merge existing runs |

## Parameters in the app

- **List size**: the number of bars, from 5 to 200 (default 40).
- **Data**: Random, Nearly sorted, Reversed or Few unique. The running time barely changes, but sorted input needs fewer comparisons.
- **Randomize list**: a new arrangement, shared by every sorting viewport.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter. Red marks the two front values being compared, amber a write, green values placed by the final merge; bars outside the current call are faded.
- **Recursion rows**: one row per open call under the bars, outermost on top; the bright row is the active merge, its halves in two colors.
- **Sound**: a tone per step, pitch 150 + value · 650 Hz.
- **Speed**: steps per second (logarithmic, 1 to 5000); it starts so that a full run takes about fifteen seconds.
- **Step**: scrub through the recorded steps. Restart, Step back, Play/Pause, Step and Finish, plus Space and the arrow keys.
- The status line counts steps, comparisons and writes, and shows the recursion depth.

## Numerical notes and pitfalls

- **Ties decide stability.** Taking from the left half only when `L[i] < R[j]` sends equal values from the right half first, which silently breaks stability. Use `L[i] <= R[j]`, as the app and the exported code do.
- **Midpoint overflow.** `(lo + hi) / 2` can overflow fixed-width integers on huge arrays; `lo + (hi − lo) / 2` cannot.
- **Allocation.** Allocating new halves in every call is slow. Allocate one buffer of size n up front, or alternate the roles of the array and the buffer between levels.
- **Small ranges.** Recursing down to single values wastes time; real implementations switch to insertion sort below about 16 to 64 values.
- **Already sorted halves.** If `a[mid − 1] <= a[mid]` the halves are already in order and the merge can be skipped, which makes sorted input linear.
- **Recursion depth** is only about log₂ n, so stack overflow is not a concern even for very large inputs.

## Applications

- External sorting: data larger than memory is sorted in chunks that fit, then merged in sequential passes over disk or tape. The Unix `sort` command works this way on big files.
- Stable library sorts: Timsort (Python, Java for objects, Android) is a natural merge sort, and C++ `std::stable_sort` is merge-based.
- Sorting linked lists, where merging needs no extra buffer and random access is unavailable.
- Counting inversions in O(n log n), used to compare rankings (Kendall tau distance).
- Parallel and distributed sorting, where independent halves are sorted on separate processors and merged.

## History

John von Neumann wrote a merge sort program for the EDVAC in 1945, one of the first programs written for a stored-program computer; it was described in a 1948 report by Herman Goldstine and von Neumann. Merging was already an established operation on punched-card collators, and external merge sorting on magnetic tape dominated commercial data processing in the 1950s and 1960s. The name says what it does: sorting by **merging** sorted runs.

## References

- Donald E. Knuth, *The Art of Computer Programming, Volume 3: Sorting and Searching*, sections 5.2.4 "Sorting by merging" and 5.4 "External sorting".
- Thomas H. Cormen, Charles E. Leiserson, Ronald L. Rivest, Clifford Stein, *Introduction to Algorithms*, section 2.3 "Designing algorithms".
- Robert Sedgewick and Kevin Wayne, *Algorithms*, 4th edition, section 2.2 "Mergesort".
- Donald E. Knuth, "Von Neumann's First Computer Program", Computing Surveys, 1970.
- Tim Peters, "listsort.txt", the description of Timsort in the CPython source.
- Wikipedia, "Merge sort".
