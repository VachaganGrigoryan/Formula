# Depth-first search

## Overview

**Depth-first search** (DFS) explores a graph by going as deep as it can before backing up. From the current vertex it steps to an unvisited neighbor, then to that neighbor's unvisited neighbor, and so on until it hits a dead end; only then does it back up to the most recent vertex that still has an unexplored neighbor. On a grid it looks like a single snake diving down one corridor at a time.

Unlike breadth-first search, DFS makes no promise about distance. It finds *a* path to the goal whenever one exists, but that path is usually winding and much longer than the shortest one. What it offers instead is a tiny memory footprint and a traversal order that exposes the structure of the graph, which is why it sits underneath topological sorting, cycle detection and connected components.

The app draws the search on a 31 x 19 grid. Draw walls, drag the start (a filled dot) and the goal (a ring), and watch the **frontier** (the cells waiting on the stack) as the search dives. The final path is the route through the discovery tree, so it is often far from straight.

## Formula

```math
visit(u): mark u visited
          for each neighbor n of u:  if n is unvisited, visit(n)
parent(n) = u   when n is first visited from u
stack order: the newest frontier cell is visited first (last in, first out)
```

| Symbol | Meaning |
| --- | --- |
| `u` | the cell being visited |
| `n` | a neighbor of `u` |
| `parent(n)` | the cell from which `n` was first reached; following these links back from the goal gives the path |

DFS has no distance formula because it does not minimize anything. The only invariant is the stack: the cell on top is always the one discovered most recently.

## Intuition

### One corridor at a time

Imagine walking a maze with a ball of string. At every junction you take the first unexplored branch and keep going. When you hit a dead end you wind the string back to the last junction that still has an untried branch. That is DFS, and the string is the stack.

### Why the path is rarely short

DFS stops the moment it reaches the goal, along whatever route the dive happened to take. A short path might exist one corridor over, but DFS never compared the two. In the app the neighbors are tried in the order up, right, down, left, so on an open grid the search heads **up** first and sweeps the board before it comes back down to the goal.

### Walls and mud

Walls are missing vertices. Mud is ignored: DFS counts cells, not cost, and its path does not try to avoid mud. For weighted terrain use Dijkstra's algorithm.

## Algorithm

1. Push the start onto the stack.
2. While the stack is not empty, pop a cell `u`. If `u` was already visited, discard it and pop again.
3. Mark `u` visited and record the cell it came from as `parent(u)`. Stop if `u` is the goal.
4. Push every unvisited open neighbor of `u` onto the stack, in reverse of the order they should be tried, so the first-choice neighbor ends up on top.
5. When the goal is reached, follow `parent` links back to the start. If the stack empties first, there is no path.

```
stack = [(start, none)]
while stack:
    u, came = stack.pop()
    if u in visited: continue        # mark visited on pop
    visited.add(u); parent[u] = came
    if u == goal: break
    for n in reversed(neighbors(u)):
        if n not in visited:
            stack.push((n, u))
path = walk parent links from goal back to start
```

The app first runs the whole search and records every step as an operation (`Frontier`, `Visit`, `Path`). The viewport only replays that log, which is why stepping backwards and scrubbing are exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time | O(V + E): every vertex is visited once and every edge examined once |
| space | O(V) for the visited marks and parent links; the stack itself is at most O(V) |
| on this grid | V = 589 cells, at most 4 edges each, so at most about 2 400 steps |

On a long corridor DFS keeps only one branch of unexplored neighbors per level of depth, so it often needs far less frontier memory than breadth-first search.

## Parameters in the app

- **Empty**, **Maze**, **Random walls**: presets for the shared grid. Maze carves a new random maze on every click; Random walls scatters walls with a density of 0.28. A maze has exactly one route between any two cells, so DFS and BFS then agree.
- **Paint mud (weight 5)**: switches the brush from walls to mud. DFS ignores the weight, but Dijkstra and A* on the same grid do not.
- **Drag on the grid**: draws or erases walls (the first cell you press decides which). Drag the start (filled dot) or the goal (ring) to move it.
- **Speed**: replay speed in recorded steps per second.
- **Step**: scrubs to any point of the recording.
- **Restart**, **Step back**, **Play / Pause**, **Step**, **Finish**: transport controls. Space plays or pauses, Left and Right step one operation.
- **Legend**: wall, mud, frontier (waiting on the stack), visited, and path. Mud is a tint with an inset border that stays visible under the search colors, and the path is drawn on top of it. The cell of the latest operation gets a dark outline.
- Visited cells show the `dist` they were reached with (steps along the route DFS took, not the shortest) when the cells are large enough (18 points or more).
- The header shows the cells visited, and the path length in steps once a path is found, or reports that the goal is walled off. Compare the path length with breadth-first search on the same grid.

## Numerical notes and pitfalls

- **The path is not the shortest.** DFS finds a path, not the best one. If you need the fewest steps use breadth-first search; for the cheapest route use Dijkstra or A*.
- **Recursion depth limits.** The textbook recursive version uses one call frame per level of depth, which on a long corridor or a large graph can exhaust the call stack. Use an explicit stack, as the app does.
- **Visited on pop versus on push.** Marking a cell visited when it is pushed is cheaper, but then a cell's parent is the first cell that saw it, which is not the order a true recursive DFS would follow, and the path can differ. The app marks on pop: a cell may sit on the stack several times and the stale copies are skipped when they surface, which matches the recursive order exactly.
- **Neighbor order decides everything.** The order up, right, down, left sets which corridor is dived into first, and therefore which path is reported. Reversing the order gives a different, equally valid route.
- **Reverse before pushing.** A stack reverses order, so push the neighbors reversed if you want the first one tried first.

## Applications

- Maze generation (recursive backtracking) and maze solving.
- Topological sorting of a directed acyclic graph, by recording the order in which vertices finish.
- Cycle detection in directed and undirected graphs.
- Connected components, strongly connected components and articulation points.
- Backtracking search: Sudoku, N-queens and other puzzles explore choices depth first and undo them on a dead end.

## History

Depth-first search as a way out of a maze goes back to Charles Pierre Tremaux, a French engineer who described the method in the 19th century: mark each passage you take, and never take a marked passage twice unless you are forced to back up. In 1972-74 Robert Tarjan and John Hopcroft showed how powerful the traversal is, publishing linear-time algorithms for biconnected components, strongly connected components and planarity testing built on DFS (Hopcroft and Tarjan, 1973 and 1974).

## References

- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapter 22, "Elementary Graph Algorithms".
- R. Sedgewick and K. Wayne, *Algorithms*, 4th edition, section 4.1, "Undirected Graphs".
- J. Hopcroft and R. Tarjan, "Algorithm 447: efficient algorithms for graph manipulation", Communications of the ACM, 1973.
- J. Hopcroft and R. Tarjan, "Efficient planarity testing", Journal of the ACM, 1974.
- R. Tarjan, "Depth-first search and linear graph algorithms", SIAM Journal on Computing, 1972.
- Wikipedia, "Depth-first search".
