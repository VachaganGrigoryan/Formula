# Bubble sort

## Overview

Bubble sort orders a list by walking through it again and again, comparing each pair of **neighbours** and swapping the pair when it is out of order. After the first pass the largest value has been carried all the way to the right end; after the second pass the second largest sits next to it, and so on. The sorted region grows from the right by one value per pass, like bubbles rising to the surface.

It is the simplest exchange sort and the usual first example of a sorting algorithm. It is stable, works in place, and with an early-exit check it finishes in a single pass on input that is already sorted. Its cost, though, is quadratic on almost every other input, so in practice it is a teaching tool rather than a production sort.

## Formula

```math
pass i = 0 … n − 2:   for j = 0 … n − 2 − i:   if a_{j} > a_{j+1} then swap(a_{j}, a_{j+1})
C_{worst} = n(n − 1)/2,   C_{best} = n − 1
S = inv(a) = #{ (i, j) : i < j and a_{i} > a_{j} }
E[S] = n(n − 1)/4   for a random permutation
```

| Symbol | Meaning |
| --- | --- |
| `n` | number of values in the list |
| `a_j` | the value at index j |
| `C` | number of comparisons |
| `S` | number of swaps |
| `inv(a)` | inversions: pairs of positions whose values are in the wrong order |

Every adjacent swap of an out-of-order pair removes **exactly one inversion** and creates none, so the number of swaps bubble sort makes equals the number of inversions in the input, no matter how the passes are arranged. A random permutation has on average half of its n(n − 1)/2 pairs inverted.

## Intuition

Follow the largest value during the first pass. Every time it is compared with its right neighbour it wins, so it is swapped one step right, and it keeps moving until it reaches the end. Large values therefore travel fast (they are sometimes called *rabbits*). A small value near the right end can only move one step left per pass, because in each pass it loses exactly one comparison against the bigger value just in front of it. These slow values are called *turtles*, and they are why bubble sort needs so many passes.

Two facts make the loop bounds work:

- After pass i, the last i + 1 positions hold the i + 1 largest values in order, so pass i + 1 can stop one position earlier.
- If a whole pass makes no swap, every neighbouring pair is in order, which means the entire list is sorted and the algorithm can stop.

## Algorithm

1. Set `i = 0`.
2. Walk `j` from 0 to `n − 2 − i`. Compare `a[j]` with `a[j + 1]`; if the left one is larger, swap them and remember that a swap happened.
3. Position `n − 1 − i` now holds its final value.
4. If no swap happened in this pass, stop: the rest of the list is already sorted.
5. Otherwise increase `i` and go back to step 2.

```
bubble_sort(a):
    n = len(a)
    for i in 0 .. n-1:
        swapped = false
        for j in 0 .. n-i-2:
            if a[j] > a[j+1]:
                swap a[j], a[j+1]
                swapped = true
        if not swapped:
            break
```

## Complexity

| Measure | Value |
| --- | --- |
| Best time (already sorted) | O(n): one pass, n − 1 comparisons, 0 swaps |
| Average time | O(n²): about n²/2 comparisons, n(n − 1)/4 swaps |
| Worst time (reversed) | O(n²): n(n − 1)/2 comparisons and n(n − 1)/2 swaps |
| Extra space | O(1) |
| Stable | yes, equal neighbours are never swapped |
| In place | yes |
| Adaptive | yes, partly: the early exit rewards input that is already in order |

Swaps always equal the inversion count. Comparisons depend on the number of passes. A value with k larger values to its left moves exactly one step left per pass until all k have passed it, so the pass count is one more than the largest such k (the extra pass confirms that nothing moved). A single small turtle at the far right therefore forces about n passes even if everything else is in place.

## Parameters in the app

- **List size**: the number of bars, from 5 to 200 (default 40).
- **Data**: the input distribution: Random, Nearly sorted, Reversed or Few unique. Reversed is the worst case; Nearly sorted shows the early exit.
- **Randomize list**: a new arrangement, shared by every sorting viewport so algorithms can be compared on the same input.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter. Colors: blue unsorted, red comparing, amber swapping, green in final place.
- **Sound**: plays a tone for each step, with pitch 150 + value · 650 Hz.
- **Speed**: steps per second (logarithmic, 1 to 5000); it starts so that a full run takes about fifteen seconds.
- **Step**: scrub to any recorded step, forwards or backwards. Restart, Step back, Play/Pause, Step and Finish do what they say; Space and the arrow keys work too.
- The status line counts steps, comparisons and swaps.

## Numerical notes and pitfalls

- **Off-by-one in the inner loop.** The inner loop must stop at `n − 2 − i` so that `a[j + 1]` stays in range; forgetting the `− i` is still correct but wastes comparisons on the finished tail.
- **Strict comparison.** Swap only when `a[j] > a[j + 1]`. Using `>=` swaps equal values, which breaks stability and keeps the early exit from firing while equal values sit side by side.
- **No early exit.** Without the `swapped` flag the best case is O(n²) too; many textbook versions omit it.
- **Turtles.** Cocktail shaker sort (alternating left-to-right and right-to-left passes) and comb sort (comparing values a shrinking gap apart) were designed to move small values left faster.
- **Swap cost.** Each swap is three assignments. On real hardware bubble sort is usually slower than insertion sort, which does the same number of moves as shifts of one assignment each.

## Applications

- Teaching: loops, invariants, stability, inversions and the O(n²) barrier.
- Checking whether a list is already sorted, and fixing one or two adjacent misplacements, in a single linear pass.
- Nearly sorted data that changes a little between frames, such as the edge lists of a polygon scan-line fill in graphics.
- Odd-even transposition sort, a parallel cousin that compares disjoint neighbour pairs at once, used on mesh processors and in sorting networks.

## History

Exchange sorts were among the first sorting methods studied for electronic computers. Edward Harry Friend analysed "sorting by exchange" in 1956. The name *bubble sort* appeared in print in the early 1960s, notably in Kenneth Iverson's *A Programming Language* (1962), after earlier names such as *exchange sort* and *sinking sort*. Donald Knuth later wrote that the bubble sort "seems to have nothing to recommend it, except a catchy name and the fact that it leads to some interesting theoretical problems". Owen Astrachan traced the algorithm's history in 2003.

## References

- Donald E. Knuth, *The Art of Computer Programming, Volume 3: Sorting and Searching*, section 5.2.2, "Sorting by exchanging".
- Thomas H. Cormen, Charles E. Leiserson, Ronald L. Rivest, Clifford Stein, *Introduction to Algorithms*, problem 2-2 "Correctness of bubblesort".
- Owen Astrachan, "Bubble Sort: An Archaeological Algorithmic Analysis", SIGCSE 2003.
- Robert Sedgewick and Kevin Wayne, *Algorithms*, 4th edition, chapter 2.1 "Elementary Sorts".
- Wikipedia, "Bubble sort".
