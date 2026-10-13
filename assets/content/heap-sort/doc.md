# Heap sort

## Overview

Heap sort is selection sort with a better data structure. Selection sort scans the whole unsorted part to find its maximum, which costs O(n) per value. Heap sort first rearranges the array into a **max-heap**, a binary tree stored in the array itself in which every parent is at least as large as its children. The maximum is then always at the root, and removing it and repairing the heap costs only O(log n).

The result is a sort that is O(n log n) in every case and needs no memory beyond a few variables. Those guarantees make it the safety net of practical sorting: introsort falls back to heap sort when quick sort recurses too deeply. Its weaknesses are that it is not stable and that it jumps around the array, so it uses caches poorly and is usually slower than a good quick sort.

## Formula

```math
left(i) = 2i + 1,   right(i) = 2i + 2,   parent(i) = floor((i − 1) / 2)
heap property:  a[parent(i)] ≥ a[i]   for 0 < i < m
height = floor(log_{2} m)
build:  Σ_{h ≥ 0} (n / 2^{h+1}) · 2h = 2n comparisons at most
extract:  Σ_{m=2}^{n} 2 log_{2} m ≤ 2n log_{2} n comparisons
```

| Symbol | Meaning |
| --- | --- |
| `i` | an index in the array, read as a node of the tree |
| `m` | current heap size; `a[0..m-1]` is the heap, `a[m..n-1]` is sorted |
| `h` | height of a node above the bottom level of the tree |
| `n` | number of values |

Indices here are 0-based. With 1-based arrays the children are `2i` and `2i + 1` and the parent is `floor(i / 2)`.

## Intuition

The array *is* the tree: read it level by level, left to right. Slot 0 is the root, slots 1 and 2 its children, slots 3 to 6 the grandchildren, and so on. No pointers are stored; the index arithmetic above is the whole structure, and a heap of n values is always a complete tree of height floor(log₂ n).

**Sift-down** repairs a single misplaced node: compare it with its two children, and if the larger child is bigger, swap them and continue from the child's slot. It stops at a leaf or when both children are smaller, after at most one swap and two comparisons per level.

**Building** the heap sifts down every parent, from the last one, `n/2 - 1`, back to the root. When a node is sifted, both of its subtrees are already heaps. Building is linear, not n log n: half the nodes are leaves and need no work, a quarter sit one level up and sink at most one level, and so on. The sum of `h / 2^(h+1)` over all heights h converges to 1.

**Extracting**: the root is the largest value in the heap, so swap it with the last heap slot, where it belongs in the sorted output. The heap shrinks by one, and the value that just landed on the root is sifted down. After n − 1 extractions the array is sorted, smallest first.

## Algorithm

1. For `i` from `n/2 - 1` (integer division) down to `0`: sift down `a[i]` in a heap of size `n`.
2. For `end` from `n - 1` down to `1`: swap `a[0]` and `a[end]`, then sift down `a[0]` in a heap of size `end`.

```
sift_down(a, i, m):                 # the heap is a[0..m-1]
    loop:
        largest = i
        for c in (2i + 1, 2i + 2):
            if c < m and a[c] > a[largest]: largest = c
        if largest == i: return
        swap a[i], a[largest]
        i = largest

heap_sort(a):
    n = len(a)
    for i in n/2 - 1 down to 0:  sift_down(a, i, n)
    for end in n - 1 down to 1:  swap a[0], a[end]; sift_down(a, 0, end)
```

## Complexity

| Case | Time | Notes |
| --- | --- | --- |
| Best | O(n log n) | for distinct values; O(n) if all values are equal |
| Average | O(n log n) | about 2n log₂ n comparisons |
| Worst | O(n log n) | at most about 2n log₂ n + 2n comparisons |
| Space | O(1) | sift-down written as a loop |
| Stable | No | swapping the root to the end reorders equal values |
| In place | Yes | the heap lives inside the array |

Comparisons: at most 2n while building and two per level per sift-down while extracting. Swaps: at most one per level, about n log₂ n in total. Because the value moved to the root came from the bottom of the heap, it usually sinks almost all the way back down, which is why the average is close to the worst case.

## Parameters in the app

- **List size** (5 to 200) and **Data** (Random, Nearly sorted, Reversed, Few unique): the input, shared by every sorting page. Heap sort's step count depends only mildly on the order of the input (a reversed list is already a max-heap). **Randomize list** draws a new arrangement.
- **Draw as** (Bars, Radial, Color spectrum, Dot scatter) and **Sound** (a tone per value touched).
- **Speed**, **Step**, and the Restart, Step back, Play / Pause, Step and Finish buttons (or Space and the arrow keys) replay the recorded steps.
- **Tree view**: with 63 values or fewer, the heap is also drawn as a binary tree under the bars, with node `i` connected to `2i + 1` and `2i + 2`. Edges inside the heap are drawn stronger than those leading to sorted values.
- Colors: blue unsorted, red a child compared with the largest value found so far, amber a swap, green in final place. Sorted values collect on the right; the largest is placed first.

## Numerical notes and pitfalls

- **Index conventions**: mixing the 0-based children `2i + 1, 2i + 2` with the 1-based `2i, 2i + 1` silently breaks the heap.
- **Build bottom-up**: sifting down from the root first does not produce a heap. Start at the last parent, `n/2 - 1`.
- **Swap with the larger of the two children**, not the first child that beats the parent; otherwise the other child ends up below a smaller parent.
- **Shrink the heap before sifting**, or the maximum just parked at the end gets pulled back in.
- **Recursion**: a recursive sift-down is tail recursive; write it as a loop to keep the space O(1).
- **Stability**: if equal keys must keep their order, sort (key, original index) pairs.
- **NaN** compares false with everything, so a NaN can sit in the heap above larger values. Remove NaNs or use a total order.
- **Speed**: the same O(n log n) as quick sort, but often two to three times slower on large arrays because of cache misses.

## Applications

- **Priority queues**: the binary heap behind Dijkstra's shortest paths, A* search, event-driven simulation, process schedulers and Huffman coding.
- **Introsort**: C++ `std::sort` switches to heap sort when quick sort's recursion gets too deep, guaranteeing O(n log n).
- **Kernels and embedded systems**: the Linux kernel's generic `sort()` is a heap sort, chosen for its O(1) memory and lack of recursion.
- **Top-k and partial sorting**: a heap of size k finds the k largest of n values in O(n log k); `std::partial_sort` uses this.

## History

J. W. J. Williams introduced both the binary heap and heap sort in 1964, as Algorithm 232 in Communications of the ACM; the name "heap" is his. The same year, Robert W. Floyd published "Treesort 3" (Algorithm 245), which builds the heap in place and bottom-up in linear time, the form used today. Later refinements include Dijkstra's smoothsort (1981), which approaches linear time on nearly sorted input, and Ingo Wegener's bottom-up heap sort (1993), which cuts the average to about n log₂ n comparisons. The heap data structure is unrelated to the "heap" of dynamic memory allocation.

## References

- J. W. J. Williams, "Algorithm 232: Heapsort", Communications of the ACM 7(6), 1964.
- R. W. Floyd, "Algorithm 245: Treesort 3", Communications of the ACM 7(12), 1964.
- T. H. Cormen, C. E. Leiserson, R. L. Rivest, C. Stein, Introduction to Algorithms, chapter 6 "Heapsort".
- D. E. Knuth, The Art of Computer Programming, Volume 3: Sorting and Searching, section 5.2.3.
- R. Sedgewick and K. Wayne, Algorithms, 4th edition, section 2.4 "Priority Queues".
- Wikipedia, "Heapsort".
