# Sorting race

## Overview

The sorting race runs several algorithms side by side on **identical copies of the same list**. Every lane replays its algorithm's recorded steps at the same number of steps per second, so the lane that needs the fewest operations finishes first. Live bars beside each lane count comparisons and moves, and when a lane finishes it also shows a real wall-clock measurement of that algorithm on this list.

The point is to compare algorithms **fairly**. Labels such as O(n log n) hide constants and say nothing about a particular input; a race on the same data, repeated across sizes and input shapes, shows how the counts grow, which algorithms adapt to order in the input, and where counts and real running time disagree.

## Formula

```math
steps = comparisons + swaps + writes + bookkeeping
finish time = steps / speed
speed_{start} = (steps of the slowest lane) / 15 s
C_{min}(n) = ceil(log₂(n!)) ≈ n log₂ n − 1.44 n
```

| Symbol | Meaning |
| --- | --- |
| `steps` | length of an algorithm's recorded log on this list |
| `bookkeeping` | marker steps: a value becoming final, a new pivot, minimum or key, a recursive call starting or ending, counting sort's tallies |
| `speed` | steps per second, the same for every lane |
| `C_min(n)` | the fewest comparisons any comparison sort needs in the worst case |

Since all lanes advance at the same speed, finishing order is simply the order of step counts. The last line is the information-theoretic lower bound: a comparison sort must distinguish all n! orderings, and each comparison has two outcomes. Counting sort escapes it because it never compares two values.

## Intuition

### Counting operations versus timing

Operation counts are **deterministic and machine-independent**: the same list always gives the same counts, on any computer. They show growth directly; doubling n roughly quadruples the steps of a quadratic sort and slightly more than doubles those of an n log n sort.

Counts treat every operation as equal, but real costs are not. A comparison that hits the cache costs a nanosecond, one that misses can cost a hundred times more. Branch mispredictions, memory allocation, recursion overhead and how many values move at once all matter. That is why the race also reports **measured time**: heap sort shares merge sort's n log n growth but jumps around memory and is usually slower, and quick sort usually wins on time despite making more comparisons than merge sort, because its inner loop is tiny and scans memory in order.

### Input distributions

- **Random**: a uniform shuffle (Fisher–Yates), the average case.
- **Nearly sorted**: sorted except for two or three values swapped out of place. Insertion and bubble sort become nearly linear; quick sort with a last-element pivot degrades towards quadratic.
- **Reversed**: the most inversions possible, n(n − 1)/2. The worst case for bubble and insertion sort, and quadratic for a last-element pivot.
- **Few unique**: only four distinct values. Many equal keys make plain 2-way quick sort lopsided, while 3-way quick sort and counting sort get faster.

## Algorithm

1. Generate one list; give every racer its own copy.
2. Run each algorithm once on its copy, recording every comparison, swap, write and marker as a step.
3. Set the speed so the longest log takes about fifteen seconds.
4. Each frame, advance a shared race clock and move every lane to step `floor(clock · speed)` of its own log.
5. A lane finishes when its log runs out; its place is 1 plus the number of lanes with fewer steps (ties share a place).
6. Separately, time each algorithm on the same list **without recording**: repeat the sort until about 4 ms have passed (at least 3 and at most 500 runs) and report the mean in microseconds per sort.

### Benchmark methodology

- Sort fresh copies of the same input; never reuse a list another algorithm has already sorted.
- Time without instrumentation, and repeat short runs so timer resolution does not dominate.
- Warm up, report the minimum or median of several trials, use an optimized build, and keep the compiler from deleting the work (the app uses `black_box`).
- Vary n (a doubling experiment reveals the growth rate) and vary the input distribution.

## Complexity

| Algorithm | Best | Average | Worst | Extra space | Stable | In place |
| --- | --- | --- | --- | --- | --- | --- |
| Bubble | n | n² | n² | 1 | yes | yes |
| Selection | n² | n² | n² | 1 | no | yes |
| Insertion | n | n² | n² | 1 | yes | yes |
| Merge | n log n | n log n | n log n | n | yes | no |
| Quick (last-element pivot) | n log n | n log n | n² | log n to n | no | yes |
| Quick (3-way) | n | n log n | n² | log n to n | no | yes |
| Heap | n log n | n log n | n log n | 1 | no | yes |
| Counting | n + k | n + k | n + k | n + k | yes | no |

All entries are O(·); `k` is the range of values for counting sort. Approximate counts on random input of size n:

| Algorithm | Comparisons | Moves |
| --- | --- | --- |
| Bubble | about n²/2 | about n²/4 swaps (the inversions) |
| Selection | exactly n(n − 1)/2 | at most n − 1 swaps |
| Insertion | about n²/4 | about n²/4 writes |
| Merge | about n log₂ n | n log₂ n writes |
| Quick | about 1.39 n log₂ n | about n ln n swaps with Lomuto partitioning |
| Heap | about 2 n log₂ n | about n log₂ n swaps |
| Counting | 0 | 2n writes, plus n + k tally steps |

## Parameters in the app

- **Racers**: check any of the eight algorithms; lanes appear in catalogue order (default: bubble, insertion, merge, quick, heap).
- **List size**: 5 to 200 values (default 40).
- **Data**: Random, Nearly sorted, Reversed or Few unique.
- **Randomize list**: a new arrangement, shared with every sorting viewport.
- **Draw as**: Bars, Radial, Color spectrum or Dot scatter, using the same colors as the single-algorithm pages.
- **Speed**: steps per second for every lane (logarithmic, 1 to 5000), paced at first so the slowest lane takes about fifteen seconds.
- **Go / Pause / Race again**, **Restart** and **Finish** control the race clock.
- Each lane shows its place (or progress), its finish time at this speed, and bars for comparisons, swaps + writes, and measured microseconds, each scaled to the largest value in the race.

## Numerical notes and pitfalls

- **Steps include bookkeeping.** Markers such as "this value is final" or a merge call starting count as steps, so finishing order can differ slightly from comparisons + moves alone. Compare the metric bars, not just the places.
- **Small n hides growth.** For a dozen values a quadratic sort can beat every n log n one; real libraries exploit this by switching to insertion sort on short ranges.
- **Timing noise.** Measured microseconds vary with CPU frequency scaling, other processes and caches; trust ratios and trends over single numbers.
- **Adversarial inputs.** Sorted and reversed input push a last-element-pivot quick sort to n² steps and n-deep recursion; library versions pick pivots by median-of-three or at random.
- **Equal keys.** Few-unique data separates algorithms that handle duplicates well (3-way quick, counting) from those that do not.

## Applications

- Choosing a sort for a workload: small arrays, nearly sorted logs, data with many duplicates, or memory-limited devices.
- Teaching asymptotic growth with concrete counts rather than formulas.
- Tuning hybrid sorts, such as the cutoff where introsort or Timsort hands short ranges to insertion sort.

## History

Counting the operations of sorts goes back to the 1950s and was made systematic by Donald Knuth's *The Art of Computer Programming* (1973), which gives exact average counts for each method. Racing them visually was popularised by Ronald Baecker's 1981 film *Sorting Out Sorting*, made at the University of Toronto, which ends with nine algorithms sorting the same data side by side. Jon Bentley and Douglas McIlroy's 1993 paper on engineering a library sort set the pattern for benchmark suites that test many input shapes.

## References

- Donald E. Knuth, *The Art of Computer Programming, Volume 3: Sorting and Searching*, chapter 5.
- Robert Sedgewick and Kevin Wayne, *Algorithms*, 4th edition, section 1.4 "Analysis of Algorithms" and chapter 2 "Sorting".
- Jon L. Bentley and M. Douglas McIlroy, "Engineering a Sort Function", Software: Practice and Experience, 1993.
- Ronald M. Baecker, *Sorting Out Sorting*, film, University of Toronto, 1981.
- Wikipedia, "Sorting algorithm".
