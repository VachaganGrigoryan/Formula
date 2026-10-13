# Selection sort

## Overview

Selection sort builds the sorted list from the left, one value at a time. On each round it **scans the unsorted part for its smallest value**, then swaps that value into the first unsorted position. After round i the first i + 1 positions hold the i + 1 smallest values in order, and they never move again.

Its comparison count is fixed at n(n − 1)/2 whatever the input, so it is quadratic even on a sorted list. What it does well is **move data rarely**: it makes at most n − 1 swaps, the fewest of the simple sorts. That matters when a write is far more expensive than a read, and the same select-the-extreme idea, with a heap instead of a linear scan, becomes heap sort.

## Formula

```math
for i = 0 … n − 2:   m = argmin_{i ≤ j < n} a_{j},   swap(a_{i}, a_{m})
C(n) = (n − 1) + (n − 2) + … + 1 = n(n − 1)/2
S(n) ≤ n − 1
```

| Symbol | Meaning |
| --- | --- |
| `n` | number of values in the list |
| `i` | the first unsorted position; everything left of it is final |
| `m` | index of the smallest value in `a[i..n-1]` (the purple bar) |
| `C(n)` | number of comparisons |
| `S(n)` | number of swaps |

The comparison count is an exact sum: round i compares the running minimum with each of the n − 1 − i values after position i. It does not depend on the order of the input at all.

## Intuition

Think of laying out a hand of cards on a table by repeatedly picking the lowest remaining card and putting it at the end of a row. Each pick needs a look at every card still in your hand, which is where the n²/2 comparisons come from, but each card is moved exactly once.

The invariant is simple: positions `0 .. i-1` hold the i smallest values in sorted order, and every value at or after `i` is at least as large as all of them. Finding the minimum of the rest and swapping it into position `i` extends the invariant by one. After n − 1 rounds the last value is necessarily the largest, so the list is sorted.

During a scan the running minimum changes only when a smaller value turns up. On random data that happens rarely (about ln m times in a scan over m values), so the purple bar jumps a few times early in each scan and then sits still.

## Algorithm

1. For `i` from 0 to `n − 2`:
2. Set `m = i`. For each `j` from `i + 1` to `n − 1`, compare `a[j]` with `a[m]`; if `a[j]` is smaller, set `m = j`.
3. If `m != i`, swap `a[i]` and `a[m]`.
4. Position `i` is now final.

```
selection_sort(a):
    n = len(a)
    for i in 0 .. n-2:
        m = i
        for j in i+1 .. n-1:
            if a[j] < a[m]:
                m = j
        if m != i:
            swap a[i], a[m]
```

## Complexity

| Measure | Value |
| --- | --- |
| Best time | O(n²): still n(n − 1)/2 comparisons, 0 swaps on sorted input |
| Average time | O(n²): n(n − 1)/2 comparisons, fewer than n swaps |
| Worst time | O(n²): n(n − 1)/2 comparisons, n − 1 swaps |
| Extra space | O(1) |
| Stable | no, a long-distance swap can jump a value over an equal one |
| In place | yes |
| Adaptive | no, the input order does not change the comparison count |

Compared with bubble sort on random data it makes the same number of comparisons in the worst case but about n swaps instead of about n²/4, so it usually runs faster. Compared with insertion sort it does roughly twice as many comparisons on average and gains nothing from sorted input.

## Parameters in the app

- **List size**: the number of bars, from 5 to 200 (default 40).
- **Data**: Random, Nearly sorted, Reversed or Few unique. Watch the comparison count: it is the same for all four.
- **Randomize list**: a new arrangement, shared by every sorting page.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter. Colors: blue unsorted, red comparing, amber swapping, purple the current minimum, green in final place.
- **Sound**: a tone per step, pitch 150 + value · 650 Hz.
- **Speed**: steps per second (logarithmic, 1 to 5000); it starts so that a full run takes about fifteen seconds.
- **Step**: scrub through the recorded steps. Restart, Step back, Play/Pause, Step and Finish, plus Space and the arrow keys.
- The status line counts steps, comparisons and swaps.

## Numerical notes and pitfalls

- **Instability.** Sorting `[2a, 2b, 1]` swaps `2a` with `1`, giving `[1, 2b, 2a]`: the two equal keys changed order. A stable variant inserts the minimum by shifting the values in between instead of swapping, at the cost of O(n²) moves.
- **Use a strict comparison.** `a[j] < a[m]` keeps the first of several equal minima; `<=` picks the last and makes more pointless updates.
- **Self-swaps.** Skipping the swap when `m == i` saves writes; without the check the algorithm always makes exactly n − 1 swaps.
- **The last round.** The outer loop can stop at `n − 2`; a one-element suffix is already in place.
- **Double-ended selection.** Finding the minimum and maximum in the same scan halves the number of rounds but not the order of growth.

## Applications

- Memory where writes are slow or wear the medium out, such as flash or EEPROM, where a sort with at most n − 1 swaps is attractive (cycle sort goes further, writing each value at most once).
- Very small arrays inside embedded code, where its short, branch-light loop and zero extra memory are all that matter.
- Picking only the k smallest values: k rounds of selection cost O(nk), fine when k is tiny.
- Teaching the select-the-extreme idea that tournament sort and heap sort speed up to O(n log n).

## History

Selection is the method people use by hand, and it is folklore rather than an invention. It was among the internal sorting methods discussed in the first studies of computer sorting in the 1950s. Knuth's *The Art of Computer Programming* treats it as *straight selection* and shows how replacing the linear scan with a tournament tree, and then with a heap (J. W. J. Williams, 1964), turns the same idea into an O(n log n) sort. The name describes the core step: repeatedly **select** the smallest remaining value.

## References

- Donald E. Knuth, *The Art of Computer Programming, Volume 3: Sorting and Searching*, section 5.2.3, "Sorting by selection".
- Robert Sedgewick and Kevin Wayne, *Algorithms*, 4th edition, chapter 2.1 "Elementary Sorts".
- Thomas H. Cormen, Charles E. Leiserson, Ronald L. Rivest, Clifford Stein, *Introduction to Algorithms*, exercise 2.2-2.
- J. W. J. Williams, "Algorithm 232: Heapsort", Communications of the ACM, 1964.
- Wikipedia, "Selection sort".
