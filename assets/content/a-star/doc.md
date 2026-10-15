# A* search

## Overview

**A\* search** (pronounced "A star") finds the cheapest path from a start to a goal like Dijkstra's algorithm does, but it is steered by a **heuristic** `h(n)`: a cheap guess of how much cost remains from cell `n` to the goal. Instead of settling cells in order of the cost already paid, it settles them in order of the cost paid plus the guess. Cells that point toward the goal are tried first, so the search wanders far less.

On the app's grid the guess is the **Manhattan distance** to the goal, the number of horizontal plus vertical steps ignoring walls and mud. Entering a cell costs 1, or 5 for mud. With a good heuristic A* returns the same optimal cost as Dijkstra while visiting far fewer cells, which you can see by running both on the same grid and comparing the **visited** count in the header.

The board shows the start as a filled dot and the goal as a ring. When cells are large enough (18 points or more) each visited cell shows its cost so far, `g`.

## Formula

```math
f(n) = g(n) + h(n)
g(n) = cost of the cheapest known path from the start to n
h(n) = |x(n) - x(goal)| + |y(n) - y(goal)|     (Manhattan distance)
admissible: h(n) <= true remaining cost      consistent: h(u) <= w(u, v) + h(v)
```

| Symbol | Meaning |
| --- | --- |
| `g(n)` | the cost already paid to reach `n` |
| `h(n)` | the estimate of the cost still to pay from `n` to the goal |
| `f(n)` | the estimated total cost of the best path that goes through `n` |
| `w(u, v)` | the cost of entering `v`: 1, or 5 on mud |

A heuristic is **admissible** if it never overestimates the true remaining cost, and **consistent** if it never drops by more than the cost of one step. Manhattan distance is both on a 4-connected grid, because every step costs at least 1 and moves the distance by at most 1. Consistency means the first time a cell is settled its `g` is already optimal, so no cell is ever expanded twice.

## Intuition

### Dijkstra with a compass

Dijkstra spreads out in every direction like a ripple. A* adds a compass: the guess `h` makes cells toward the goal look cheaper, so the ripple stretches into an ellipse aimed at the goal. If `h` is always 0 the compass is switched off and A* is exactly Dijkstra.

### Why optimality survives

The guess never overestimates, so the true best path always has an `f` value no larger than its real cost. The search cannot settle the goal by a worse path while a better one is still waiting with a lower `f`.

### Mud and walls

The heuristic ignores both. Mud makes the true cost higher than `h`, which is fine because `h` only has to be a lower bound. A wall across the way can make A* spend a lot of effort in the pocket facing the goal before it backs out, which is the weakness of any heuristic that cannot see obstacles.

## Algorithm

1. Set `g(start) = 0` and put the start in a priority queue keyed by `f = g + h`.
2. Remove the cell `u` with the smallest `f`; on a tie take the one with the smaller `h`. Skip it if it is stale or already settled. Stop if `u` is the goal.
3. For each open neighbor `v`, compute `alt = g(u) + w(v)`. If `alt < g(v)`, set `g(v) = alt`, remember `parent(v) = u`, and push `v` with key `alt + h(v)`.
4. When the goal is settled, follow `parent` links back to the start. If the queue empties first, there is no path.

```
g = {start: 0}; heap = [(h(start), h(start), start)]
while heap:
    f, hu, u = heap.pop_min()
    if u in done or f != g[u] + hu: continue     # stale entry
    done.add(u)
    if u == goal: break
    for v in neighbors(u):
        alt = g[u] + weight(v)
        if alt < g.get(v, inf):
            g[v] = alt; parent[v] = u
            heap.push((alt + h(v), h(v), v))
path = walk parent links from goal back to start
```

The app first runs the whole search and records every step as an operation (`Frontier` with a cost, `Visit`, `Path`). The viewport only replays that log, which is why stepping backwards and scrubbing are exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time | worst case like Dijkstra, O((V + E) log V); in practice far fewer cells are expanded |
| space | O(V) for the `g` table, parent links and heap |
| on this grid | V = 589 cells; on an open grid A* visits only a narrow band of cells between start and goal |

How much A* saves depends entirely on how close `h` is to the true cost. A perfect heuristic expands only the cells of one optimal path; a useless one (`h = 0`) expands as many as Dijkstra.

## Parameters in the app

- **Empty**, **Maze**, **Random walls**: presets for the shared grid. Maze carves a new random maze on every click; Random walls scatters walls with a density of 0.28.
- **Paint mud (weight 5)**: switches the brush from walls to mud. A* weighs the mud against the detour just as Dijkstra does, and still finds a path of the same cost.
- **Drag on the grid**: draws or erases walls (the first cell you press decides which). Drag the start (filled dot) or the goal (ring) to move it.
- **Speed**: replay speed in recorded steps per second.
- **Step**: scrubs to any point of the recording.
- **Restart**, **Step back**, **Play / Pause**, **Step**, **Finish**: transport controls. Space plays or pauses, Left and Right step one operation.
- **Legend**: wall, mud, frontier (waiting in the priority queue), visited, and path. Mud is a tint with an inset border that stays visible under the search colors, and the path is drawn on top of it. The cell of the latest operation gets a dark outline.
- Visited cells show their `dist` (cost so far) when the cells are large enough (18 points or more).
- The header shows the cells visited, and once a path is found its length in steps and its total **cost**. Compare **visited** with the Dijkstra tool on the same grid.

## Numerical notes and pitfalls

- **An inadmissible heuristic loses optimality.** If `h` overestimates, A* can settle the goal by a worse path while a cheaper one is still waiting. Multiplying Manhattan distance by 2, for instance, makes the search faster and greedier but no longer exact. Diagonal moves would need the Chebyshev or octile distance instead.
- **Inconsistent heuristics re-expand cells.** An admissible heuristic that is not consistent is still optimal if cells can be reopened, but a closed set that never reopens can then return a wrong answer.
- **Tie-breaking affects work.** On a uniform grid many cells share the same `f`. Preferring the one with the smaller `h` (closest to the goal) makes the search commit to a direct line instead of fanning out over every tie. The app breaks ties that way.
- **Skip stale heap entries.** As in Dijkstra, improvements push new entries and the old ones are ignored when they surface.
- **The heuristic must match the costs.** Manhattan distance is a valid lower bound only because the cheapest step costs 1. If open cells cost less than 1, scale `h` down accordingly.

## Applications

- Video games: unit movement and non-player character pathfinding on tile maps.
- Navigation meshes and road maps, often with a straight-line distance as `h`.
- Puzzle solving, such as the 15-puzzle, where `h` is the sum of the tiles' Manhattan distances to their places.
- Robot motion planning and route planning.
- Parsing and planning problems in AI, wherever a good estimate of the remaining cost exists.

## History

A* was published in 1968 by Peter Hart, Nils Nilsson and Bertram Raphael at the Stanford Research Institute (SRI), as part of the Shakey the robot project, which needed a way for a mobile robot to plan routes through rooms. Their paper, "A Formal Basis for the Heuristic Determination of Minimum Cost Paths", proved that an admissible heuristic keeps the result optimal, generalizing Dijkstra's algorithm with a goal-directed guess.

## References

- P. E. Hart, N. J. Nilsson and B. Raphael, "A Formal Basis for the Heuristic Determination of Minimum Cost Paths", IEEE Transactions on Systems Science and Cybernetics, 1968.
- S. Russell and P. Norvig, *Artificial Intelligence: A Modern Approach*, chapter 3, "Solving Problems by Searching".
- Amit Patel, "Introduction to A*", Red Blob Games.
- J. Pearl, *Heuristics: Intelligent Search Strategies for Computer Problem Solving*, 1984.
- Wikipedia, "A* search algorithm".
