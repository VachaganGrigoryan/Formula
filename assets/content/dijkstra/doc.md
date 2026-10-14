# Dijkstra's algorithm

## Overview

**Dijkstra's algorithm** finds the cheapest path from a start vertex to every other vertex in a graph whose edges have non-negative costs. It keeps a tentative cost `dist(v)` for each vertex and repeatedly settles the unsettled vertex with the smallest one. Once a vertex is settled its cost is final.

On the app's grid every cell is a vertex. Entering an ordinary cell costs 1 and entering a **mud** cell costs 5, so the cheapest route is not always the one with the fewest steps: if a detour costs less than wading through mud, Dijkstra takes the detour. Breadth-first search, which counts steps only, cannot do that.

The app draws the search on a 31 x 19 grid with the start as a filled dot and the goal as a ring. Cells are settled in order of total cost, so the visited region grows as a blob that bulges where the terrain is cheap and stalls at mud. When cells are large enough (18 points or more) each visited cell shows its final `dist` value.

## Formula

```math
dist(s) = 0,   dist(v) = infinity otherwise
dist(v) = min(dist(v), dist(u) + w(u, v))     (relaxation)
settle the unsettled vertex with the smallest dist
```

| Symbol | Meaning |
| --- | --- |
| `s` | the start cell |
| `u` | the cell just settled (taken from the priority queue) |
| `v` | a neighbor of `u` |
| `w(u, v)` | the cost of entering `v`: 1, or 5 on mud |
| `dist(v)` | the cheapest known total cost from `s` to `v` |

**Relaxation** asks whether going through `u` beats the best route found so far to `v`. When the smallest tentative cost is settled, no later route can beat it, because every remaining edge has a non-negative cost. That is the whole correctness argument, and it is exactly what a negative edge would break.

## Intuition

### A wavefront that slows in mud

Picture water spreading from the start where mud is thick syrup: it crosses a mud cell five times more slowly than a clear one. Dijkstra settles cells in the order the water arrives. The first time the water reaches the goal is the cheapest path.

### Why settled costs are final

Suppose a cheaper route to the settled cell `u` existed. It would have to leave the settled region through some other cell with a tentative cost below `dist(u)`, but then that cell would have been settled first. So the smallest tentative cost is always already optimal.

### Unweighted graphs

If every weight is 1 the priority queue degenerates into a first-in, first-out queue and Dijkstra visits cells in the same ring order as breadth-first search.

## Algorithm

1. Set `dist(start) = 0` and put the start in a priority queue keyed by `dist`.
2. Remove the cell `u` with the smallest key. If `u` is already settled, skip it (it is a stale entry). Otherwise settle `u`. Stop if `u` is the goal.
3. For each open neighbor `v` of `u`, compute `alt = dist(u) + w(v)`. If `alt < dist(v)`, set `dist(v) = alt`, remember `parent(v) = u`, and push `v` with key `alt`.
4. When the goal is settled, follow `parent` links back to the start. If the queue empties first, there is no path.

```
dist = {start: 0}; heap = [(0, start)]
while heap:
    d, u = heap.pop_min()
    if u in done: continue           # stale entry
    done.add(u)
    if u == goal: break
    for v in neighbors(u):
        alt = d + weight(v)
        if alt < dist.get(v, inf):
            dist[v] = alt; parent[v] = u
            heap.push((alt, v))
path = walk parent links from goal back to start
```

The app first runs the whole search and records every step as an operation (`Frontier` with a cost, `Visit`, `Path`). The viewport only replays that log, which is why stepping backwards and scrubbing are exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time | O((V + E) log V) with a binary heap |
| space | O(V) for the `dist` table, parent links and heap (lazy deletion lets the heap hold up to E entries) |
| on this grid | V = 589 cells, at most 4 edges each; a few thousand heap operations at most |

With a Fibonacci heap the bound improves to O(E + V log V), but a binary heap is simpler and faster in practice on grids.

## Parameters in the app

- **Empty**, **Maze**, **Random walls**: presets for the shared grid. Maze carves a new random maze on every click; Random walls scatters walls with a density of 0.28.
- **Paint mud (weight 5)**: switches the brush from walls to mud. Paint a patch across the direct route and watch Dijkstra walk around it when the detour is cheaper than the mud.
- **Drag on the grid**: draws or erases walls (the first cell you press decides which). Drag the start (filled dot) or the goal (ring) to move it.
- **Speed**: replay speed in recorded steps per second.
- **Step**: scrubs to any point of the recording.
- **Restart**, **Step back**, **Play / Pause**, **Step**, **Finish**: transport controls. Space plays or pauses, Left and Right step one operation.
- **Legend**: wall, mud, frontier (waiting in the priority queue), visited, and path. The cell of the latest operation gets a dark outline.
- The header shows the cells visited, the path length in steps, and the total **cost** of the path. Visited cells show their `dist` when the cells are large enough.

## Numerical notes and pitfalls

- **Negative weights break it.** The settled-is-final argument needs every edge cost to be at least 0. With a negative edge a settled vertex can later be improved. Use Bellman-Ford (O(VE)) instead, which also detects negative cycles.
- **Skip stale heap entries.** A binary heap cannot cheaply lower a key, so the usual trick is to push a new entry on every improvement and ignore old ones when they surface. Forgetting to skip them re-expands settled cells and can break the complexity bound.
- **Fewest steps is not cheapest.** The path length in steps can be larger than the BFS path; compare steps and cost in the header.
- **Ties.** Several equally cheap paths often exist; the neighbor order (up, right, down, left) and the heap's tie-breaking decide which one is reported, never its cost.
- **Integer costs.** Costs here are small integers, so there is no rounding issue. With floating-point weights, compare with care and never test for equality.
- **Stopping early.** Stopping when the goal is settled is correct for a single goal, but the `dist` values of unsettled cells are only upper bounds.

## Applications

- GPS and road routing, where edge costs are travel times.
- Network routing: the link-state protocols OSPF and IS-IS run Dijkstra on each router to build its forwarding table.
- Robotics and game AI: cheapest routes over terrain with different movement costs.
- Flight and logistics planning with non-negative fares or distances.
- A building block of other algorithms, such as Johnson's all-pairs shortest paths and Yen's k shortest paths.

## History

Edsger W. Dijkstra conceived the algorithm in 1956 while working at the Mathematical Centre in Amsterdam, reportedly in about twenty minutes without pencil and paper, as a demonstration for the new ARMAC computer. He published it in 1959 as "A note on two problems in connexion with graphs", in the journal Numerische Mathematik.

## References

- E. W. Dijkstra, "A note on two problems in connexion with graphs", Numerische Mathematik 1, 1959.
- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapter 24, "Single-Source Shortest Paths".
- R. Sedgewick and K. Wayne, *Algorithms*, 4th edition, section 4.4, "Shortest Paths".
- M. L. Fredman and R. E. Tarjan, "Fibonacci heaps and their uses in improved network optimization algorithms", Journal of the ACM, 1987.
- Wikipedia, "Dijkstra's algorithm".
