# Breadth-first search

## Overview

**Breadth-first search** (BFS) explores a graph in layers. It visits the start, then every vertex one edge away, then every vertex two edges away, and so on, never moving to distance `d + 1` until all vertices at distance `d` are done. On a grid, where each cell is a vertex and each open neighbor an edge, this looks like a ring of cells spreading out from the start like a ripple on water.

Because the layers are visited in order of distance, the first time BFS reaches a vertex it has found a path with the fewest possible edges. That is why BFS is the standard answer to "what is the shortest route in steps?" on any graph whose edges all cost the same.

The app draws the search on a 31 x 19 grid. Draw walls, drag the start and the goal, and watch the **frontier** (the cells waiting in the queue) sweep outward until it touches the goal, then the final path lights up.

## Formula

```math
dist(s) = 0
dist(n) = dist(u) + 1   for each unseen neighbor n of the cell u being visited
visit cells in order of dist
queue invariant: dist(front) <= ... <= dist(back) <= dist(front) + 1
```

| Symbol | Meaning |
| --- | --- |
| `s` | the start cell |
| `u` | the cell taken from the front of the queue |
| `n` | a neighbor of `u` that has not been seen yet |
| `dist(n)` | the number of steps from `s` to `n` along the shortest path found |

The **queue invariant** is what makes the algorithm correct: at every moment the queue holds cells of at most two consecutive distances, in non-decreasing order, so removing from the front always takes a cell of the smallest distance still waiting.

## Intuition

### Rings of equal distance

Imagine dropping a stone in a pond. The ripple reaches every point at distance 1 first, then every point at distance 2. BFS does the same with a first-in, first-out queue: cells discovered earlier are expanded earlier, so the frontier is always one ring wide (two at the seam).

### Shortest in edges

Suppose a shorter path to some cell existed. Its cells would be at smaller distances, so they would all have been enqueued and expanded before the cell was reached the slower way, and the cell would already have been marked. A cell's distance is therefore final the moment it is first seen.

### Walls and mud

Walls are simply missing vertices. Mud does nothing to BFS: it counts steps, not cost, so it happily walks straight through mud. For weighted terrain, see Dijkstra's algorithm.

## Algorithm

1. Mark the start as seen with `dist = 0` and put it in the queue.
2. While the queue is not empty, remove the cell `u` at the front and visit it. Stop if `u` is the goal.
3. For each open neighbor `n` of `u` that has not been seen, mark it seen, set `dist(n) = dist(u) + 1`, remember `parent(n) = u`, and add it to the back of the queue.
4. When the goal is reached, follow `parent` links back to the start to read off the path. If the queue empties first, there is no path.

```
seen = {start}; dist[start] = 0; queue = [start]
while queue:
    u = queue.pop_front()
    if u == goal: break
    for n in neighbors(u):
        if n not in seen:
            seen.add(n)
            dist[n] = dist[u] + 1
            parent[n] = u
            queue.push_back(n)
path = walk parent links from goal back to start
```

The app first runs the whole search and records every step as an operation (`Frontier`, `Visit`, `Path`). The viewport only replays that log, which is why stepping backwards and scrubbing are exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time | O(V + E): every vertex is enqueued once and every edge examined once |
| space | O(V): the queue, the `dist` table and the `parent` links |
| on this grid | V = 589 cells, at most 4 edges each, so at most about 2 400 steps |

In the worst case the frontier holds the widest ring of the graph, which on a grid grows with the distance from the start, so BFS can use much more memory than a depth-first search.

## Parameters in the app

- **Empty**, **Maze**, **Random walls**: presets for the shared grid. Maze carves a new random maze on every click; Random walls scatters walls with a density of 0.28.
- **Paint mud (weight 5)**: switches the brush from walls to mud. BFS ignores the weight, but the other searches on the same grid do not.
- **Drag on the grid**: draws or erases walls (the first cell you press decides which). Drag the orange **S** (start) or the green **G** (goal) to move it.
- **Speed**: replay speed in recorded steps per second.
- **Step**: scrubs to any point of the recording.
- **Restart**, **Step back**, **Play / Pause**, **Step**, **Finish**: transport controls. Space plays or pauses, Left and Right step one operation.
- **Legend**: wall, mud, frontier (waiting in the queue), visited, and path. The cell of the latest operation gets a dark outline.
- The header shows the cells visited and the path length in steps, or reports that the goal is walled off.

## Numerical notes and pitfalls

- **Mark visited when enqueueing, not when dequeueing.** If a cell is only marked when it is removed from the queue, several neighbors can add it again before then. The queue fills with duplicates and the work can grow far beyond O(V + E).
- **Weighted graphs need Dijkstra.** BFS minimizes the number of edges, not their total cost. With unequal weights the fewest-step path can be much more expensive than another path; use Dijkstra's algorithm or A* instead.
- **Recursion is not BFS.** A recursive function explores one branch to the end before the next, which is depth-first. A true BFS needs an explicit first-in, first-out queue.
- **Use a real queue.** Removing from the front of an array is O(n) in many languages; use a deque or an index that moves along the list.
- **Keep `parent` links.** Recording only distances finds the length but not the route.
- **Tie-breaking changes the route, not the length.** Several shortest paths often exist; the neighbor order (up, right, down, left here) decides which one is reported.

## Applications

- Shortest paths in unweighted graphs: grids, mazes, game maps, word ladders, puzzle states.
- Social networks: degrees of separation and the people within `k` hops.
- Web crawling: pages are fetched in order of link depth from the seed.
- Flood fill and connected regions in images.
- Testing whether a graph is bipartite: color the layers alternately and look for an edge inside a layer.
- Broadcast and reachability in networks, and the level-order traversal of trees.

## History

Konrad Zuse described a breadth-first search in his 1945 PhD thesis on the Plankalkul language, though it was not published until 1972. Edward F. Moore rediscovered it in 1959 as a way to find the shortest path out of a maze. C. Y. Lee independently published it in 1961 as an algorithm for routing wires on circuit boards, which is why grid BFS is still called the Lee algorithm in chip design.

## References

- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapter 22, "Elementary Graph Algorithms".
- R. Sedgewick and K. Wayne, *Algorithms*, 4th edition, section 4.1, "Undirected Graphs".
- E. F. Moore, "The shortest path through a maze", Proceedings of an International Symposium on the Theory of Switching, 1959.
- C. Y. Lee, "An algorithm for path connections and its applications", IRE Transactions on Electronic Computers, 1961.
- Wikipedia, "Breadth-first search".
