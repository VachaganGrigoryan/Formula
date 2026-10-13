# Counting sort

## Overview

Counting sort orders integers from a small range without ever comparing two of them. It counts how many times each value occurs, turns those counts into positions with a running sum, and then copies every value straight into its slot in an output array. It runs in O(n + k) time, where k is the number of possible values, which is linear when the range is small compared with the list.

That beats the n log n lower bound for comparison sorts. There is no contradiction: the bound only limits algorithms that learn about the order by comparing pairs, and counting sort uses each value directly as an array index. The price is memory proportional to the range, and the method only works for keys that are, or can be mapped to, small integers. Done carefully it is **stable**, which makes it the building block of radix sort.

## Formula

```math
c_{v} = number of i with a_{i} = v,   for v = 0 … k − 1
C_{v} = c_{0} + c_{1} + … + c_{v}
for i = n − 1 down to 0:   v = a_{i},   C_{v} = C_{v} − 1,   out[C_{v}] = v
```

| Symbol | Meaning |
| --- | --- |
| `a_i` | the input values, integers in 0 … k − 1 |
| `n` | number of values |
| `k` | number of count slots, max(a) + 1 |
| `c_v` | tally: how many times v occurs |
| `C_v` | running sum: how many values are ≤ v, which is one past the last slot of v's block |
| `out` | output array of length n |

After the running sum, the copies of v belong in `out[C_(v-1) .. C_v - 1]`. The placement loop fills each block from its right end, decrementing `C_v` before every write.

## Intuition

If the list holds three 1s and five 2s, then without comparing anything we know the 1s occupy slots 0 to 2 and the 2s slots 3 to 7. The tally gives the size of every block, and the running sum stacks the blocks end to end, so `C_v` marks where v's block ends.

Placing the values is then pure bookkeeping. Walking the input **from right to left** and filling each block **from its right end**, the last 2 in the input lands in the last slot of the 2-block, the one before it in the slot before, and so on. Equal values therefore keep their original order. That does not matter for bare numbers, but it is essential when the values are keys of larger records, and it is exactly what radix sort needs: sort by the last digit, then stably by the one before, and the earlier passes are never undone.

## Algorithm

1. Find the largest value; allocate `count` with `max + 1` zeros and an `output` of length n.
2. **Tally**: for every value v, increase `count[v]`.
3. **Running sum**: for v from 1 to max, add `count[v - 1]` to `count[v]`.
4. **Place**: for i from n − 1 down to 0, decrease `count[a[i]]` and write `a[i]` to `output[count[a[i]]]`.
5. Copy `output` back into `a`.

```
counting_sort(a):
    k = max(a) + 1
    count = [0] * k
    output = [0] * len(a)
    for v in a:            count[v] += 1
    for v in 1 .. k - 1:   count[v] += count[v - 1]
    for i in len(a) - 1 down to 0:
        count[a[i]] -= 1
        output[count[a[i]]] = a[i]
    return output
```

## Complexity

| Case | Time | Notes |
| --- | --- | --- |
| Best | O(n + k) | every pass is a fixed-length loop |
| Average | O(n + k) | the input order does not matter |
| Worst | O(n + k) | dominated by k when the range is much larger than n |
| Space | O(n + k) | the count array and the output array |
| Stable | Yes | thanks to the right-to-left placement |
| In place | No | needs an output array as large as the input |

Comparisons between values: zero. Work: n tally increments, k − 1 additions for the running sum, n decrements and n writes into the output, plus n more writes to copy it back, so 2n writes in the app's counter.

## Parameters in the app

- **Data**: the app's lists hold the values 1 to n. Random, Nearly sorted and Reversed are permutations, so every tally is 1 and the running sum is a straight ramp. **Few unique** uses four distinct values, so the histogram has four tall bars and the output fills in four blocks; it is the most telling setting.
- **List size** (5 to 200) and **Randomize list**: the input, shared by every sorting viewport. Here k grows with the list size.
- **Draw as** (Bars, Radial, Color spectrum, Dot scatter) and **Sound** (a tone per value touched).
- **Speed**, **Step**, and the Restart, Step back, Play / Pause, Step and Finish buttons (or Space and the arrow keys) replay the recorded steps.
- Panels: the input on top, the histogram `count[value]` in the middle (tallies, then running sums, then counting down as values are placed), and the output array at the bottom.
- Colors: blue unsorted; red a value being tallied, or the running sum being added into the next slot; amber the tally being changed, the value being placed and its output slot; green values copied back into their final place. The comparison counter stays at 0.

## Numerical notes and pitfalls

- **Large ranges**: the count array has one slot per *possible* value. Sorting a thousand 32-bit integers this way would need four billion counters; use radix sort (counting sort per byte) or a comparison sort instead.
- **Negative values or a large minimum**: index by `v - min` and use k = max − min + 1.
- **Off-by-one**: the inclusive running sum points one *past* v's block, so decrement before writing. The exclusive variant, starting positions with a left-to-right walk and increment after writing, is equally correct and stable.
- **Stability**: walking left to right while filling blocks from their right ends reverses equal values, which silently breaks radix sort.
- **Keys only**: if there are no attached records, you can skip the output array and simply write each value `count[v]` times.
- **Non-integer keys** must first be mapped to small integers (bucket indices, ranks, character codes).
- **Counter width**: a count can reach n, so its type must hold n.

## Applications

- **Radix sort**: each pass of LSD radix sort is a stable counting sort on one digit or byte; 32-bit integers sort in four passes with k = 256.
- **Small keys**: ages, exam grades, ratings, days of the week, byte values, characters of a fixed alphabet.
- **Histogram equalization** in image processing uses the same tally and running sum (the cumulative histogram) to remap gray levels.
- **Text indexing**: building suffix arrays and the Burrows–Wheeler transform starts with bucket counts of characters.
- **Parallel and GPU sorting**: radix sorts on GPUs are histograms plus prefix sums (scans), the two steps of counting sort.

## History

Harold H. Seward described counting sort in his 1954 MIT master's thesis on sorting in business data processing, together with its use as the pass of a least-significant-digit radix sort. Radix sorting itself is older: Herman Hollerith's punched-card machines for the 1890 United States census, and the card sorters that followed, sorted decks one digit column at a time. The name comes from the tally step; Knuth's The Art of Computer Programming presents it as "distribution counting".

## References

- H. H. Seward, Information Sorting in the Application of Electronic Digital Computers to Business Operations, master's thesis, MIT, 1954.
- D. E. Knuth, The Art of Computer Programming, Volume 3: Sorting and Searching, section 5.2 (distribution counting).
- T. H. Cormen, C. E. Leiserson, R. L. Rivest, C. Stein, Introduction to Algorithms, chapter 8 "Sorting in Linear Time".
- R. Sedgewick and K. Wayne, Algorithms, 4th edition, section 5.1 (key-indexed counting).
- Wikipedia, "Counting sort".
