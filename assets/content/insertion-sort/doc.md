# Insertion sort

## Overview

Insertion sort grows a sorted prefix one value at a time. It **lifts out the next unsorted value** (the key), shifts every larger value in the sorted prefix one step to the right, and drops the key into the gap that opens up. It is exactly how most people sort a hand of playing cards.

It is quadratic in the worst case, but it is stable, sorts in place, needs no setup, and its running time is proportional to how far the input is from sorted. On small or nearly sorted lists it beats every O(n log n) algorithm, which is why production sorts such as Timsort and introsort switch to it for short runs.

## Formula

```math
for i = 1 … n − 1:   key = a_{i};   shift every a_{j} > key in a_{0..i−1} one step right;   a_{j+1} = key
moves M = inv(a),   comparisons C between M and M + n − 1
T(n) = Θ(n + inv(a))
E[inv(a)] = n(n − 1)/4   for a random permutation
```

| Symbol | Meaning |
| --- | --- |
| `n` | number of values in the list |
| `key` | the value being inserted (the purple outline in the app) |
| `inv(a)` | inversions: pairs of positions whose values are in the wrong order |
| `M` | number of shifts (one-step moves to the right) |
| `C` | number of comparisons |
| `T(n)` | running time |

Each shift moves a larger value past the key, which removes exactly one inversion, so the shifts add up to the inversion count. Every insertion also makes at most one extra comparison, the one that stops the shifting, which gives the bounds on C.

## Intuition

Before step i the prefix `a[0..i-1]` is sorted. The key `a[i]` belongs somewhere in it: after all values not larger than it and before all larger ones. Walking leftwards from `i − 1`, every larger value is moved one slot right, so the hole the key left behind travels left. The first value that is not larger stops the walk, and the key falls into the hole. The prefix is sorted again and one longer.

The walk is short when the key is already nearly in place. On a sorted list each key needs one comparison and no shifts, so the whole sort is a single linear scan. On a reversed list each key travels all the way to the front, the worst case. On random data a key travels on average halfway through the prefix, about n²/4 shifts in total.

## Algorithm

1. For `i` from 1 to `n − 1`:
2. Lift out `key = a[i]` and set `j = i − 1`.
3. While `j ≥ 0` and `a[j] > key`: copy `a[j]` to `a[j + 1]` and decrease `j`.
4. Write `key` into `a[j + 1]`.

```
insertion_sort(a):
    for i in 1 .. n-1:
        key = a[i]
        j = i - 1
        while j >= 0 and a[j] > key:
            a[j+1] = a[j]        # shift right; the hole moves left
            j = j - 1
        a[j+1] = key             # drop the key into the hole
```

## Complexity

| Measure | Value |
| --- | --- |
| Best time (already sorted) | O(n): n − 1 comparisons, no shifts |
| Average time | O(n²): about n²/4 comparisons and n²/4 shifts |
| Worst time (reversed) | O(n²): n(n − 1)/2 comparisons and shifts |
| Extra space | O(1): only the key |
| Stable | yes, the walk stops at the first value that is not larger |
| In place | yes |
| Adaptive | yes: time is O(n + inversions) |
| Online | yes: values can be inserted as they arrive |

A shift is one assignment, whereas a swap is three, so insertion sort moves data more cheaply than bubble sort despite making the same number of moves. The app counts each shift and each final drop as a write, so its write count is the inversion count plus n − 1.

## Parameters in the app

- **List size**: the number of bars, from 5 to 200 (default 40).
- **Data**: Random, Nearly sorted, Reversed or Few unique. Try Nearly sorted to see the linear best case.
- **Randomize list**: a new arrangement, shared by every sorting viewport.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter. The key appears as a purple outline over the hole (red while it is being compared); bars right of the sorted prefix are faded; amber marks a write; everything turns green at the end.
- **Sound**: a tone per step, pitch 150 + value · 650 Hz.
- **Speed**: steps per second (logarithmic, 1 to 5000); it starts so that a full run takes about fifteen seconds.
- **Step**: scrub through the recorded steps. Restart, Step back, Play/Pause, Step and Finish, plus Space and the arrow keys.
- The status line counts steps, comparisons and writes.

## Numerical notes and pitfalls

- **Strict comparison.** Shift only while `a[j] > key`. Using `>=` moves equal values past each other, which breaks stability and does extra work.
- **Short-circuit order.** Test `j >= 0` before reading `a[j]`; in languages without bounds checks the reverse order reads before the array.
- **Unsigned indices.** With an unsigned `j`, decrementing past 0 wraps around. Loop on `j + 1` or use a signed index, or use a sentinel (the minimum moved to `a[0]` first) to drop the bounds test entirely.
- **Swap-based versions.** Writing the inner loop as repeated swaps of `a[j]` and `a[j + 1]` is correct but triples the number of writes.
- **Binary insertion.** Finding the insertion point by binary search cuts comparisons to O(n log n), but the shifts stay quadratic; it pays off only when comparisons are expensive.

## Applications

- The base case of hybrid sorts: Timsort (Python, Java for objects) sorts short runs with binary insertion sort, and introsort implementations finish small partitions, typically below about 16 values, with insertion sort.
- Nearly sorted data, such as a list that was sorted and then had a few entries changed, or event queues that arrive almost in order.
- Keeping a small list sorted as items stream in (online insertion).
- Shell sort, which runs insertion sort over interleaved sub-lists with shrinking gaps to move values long distances early.

## History

Insertion is the oldest and most natural way to sort by hand. In the first published discussion of computer sorting, John Mauchly's 1946 Moore School lecture, binary insertion was already described. Donald Shell's 1959 generalization, Shell sort, was one of the first sorts to beat quadratic time in practice. The name comes from the operation it repeats: **insert** the next value into an already sorted list.

## References

- Donald E. Knuth, *The Art of Computer Programming, Volume 3: Sorting and Searching*, section 5.2.1, "Sorting by insertion".
- Thomas H. Cormen, Charles E. Leiserson, Ronald L. Rivest, Clifford Stein, *Introduction to Algorithms*, section 2.1 "Insertion sort".
- Jon Bentley, *Programming Pearls*, 2nd edition, column 11 "Sorting".
- Donald L. Shell, "A high-speed sorting procedure", Communications of the ACM, 1959.
- Wikipedia, "Insertion sort".
