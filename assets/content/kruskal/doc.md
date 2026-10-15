# Kruskal's algorithm

## Overview

A **minimum spanning tree** (MST) of a connected, weighted graph is a set of edges that joins every vertex, contains no cycle, and has the smallest total weight possible. **Kruskal's algorithm** builds it greedily: sort all edges from lightest to heaviest, then walk down the list and keep an edge only if it joins two vertices that are not yet connected. Edges that would close a cycle are thrown away.

At any moment the kept edges form a **forest**, many small trees that merge as the run goes on. After `n - 1` edges are kept, the forest is a single tree and the algorithm stops.

The app draws 6 to 24 random points (14 at the start) joined by Delaunay edges, and an edge's weight is its length times 100. Kruskal's run is recorded once and replayed: the edge under test turns orange, then either joins the tree (thick blue) or is rejected (dashed grey). Vertices are colored by the tree they belong to, so you can watch separate colors merge into one.

## Formula

```math
minimize  sum of w(e) over the tree edges T
|T| = n - 1
T is connected and has no cycle
accept e = (u, v) when find(u) != find(v)
```

| Symbol | Meaning |
| --- | --- |
| `n` | the number of vertices |
| `w(e)` | the weight of edge `e` (here its length times 100) |
| `T` | the set of accepted edges, the spanning tree |
| `find(v)` | the root of the tree that contains `v` |

The **cut property** is why greedy works: for any split of the vertices into two groups, the lightest edge crossing the split belongs to some MST. When Kruskal considers an edge that joins two different trees, that edge is the lightest one leaving the tree on either side, so it is safe to keep.

## Intuition

### Merging a forest

Start with every vertex alone. Each accepted edge fuses two trees into one, so exactly `n - 1` accepted edges turn `n` separate vertices into one tree. An edge inside a single tree would create a cycle, and dropping the heaviest edge of any cycle never hurts, so such edges are rejected.

### Lightest first

Because edges are tried in order of weight, the first edge that connects two trees is the cheapest way to connect them at that moment. Nothing tried later is lighter.

### Telling trees apart quickly

Asking "are these two vertices in the same tree?" for every edge would be slow if answered by searching. **Union-find** answers it by keeping one parent link per vertex, so the root of a vertex names its tree. The app colors each vertex by its root.

## Algorithm

1. Sort the edges by weight, with ties broken by the edge's position in the input.
2. Give every vertex its own set (it is its own root).
3. For each edge `(u, v)` in order, find the roots of `u` and `v`. If they differ, accept the edge and make the root of the smaller tree a child of the root of the larger tree (**union by size**). If they are equal, reject it.
4. Stop when `n - 1` edges are accepted.

```
sort edges by weight
parent[v] = v; size[v] = 1 for every v
for (u, v, w) in edges:
    ru, rv = find(u), find(v)
    if ru != rv:
        accept (u, v)
        attach the smaller of ru, rv under the larger
        if accepted == n - 1: break
```

The app records every step as an operation (`Consider`, `Accept`, `Reject`, `Union`). The viewport replays that log, which is why stepping backwards and scrubbing are exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time | O(E log E): the sort dominates; union-find adds almost nothing |
| space | O(V + E): the edge list and the union-find arrays |
| on this graph | a Delaunay graph has fewer than 3n edges, so at most about 70 edges for 24 points |

Because log E is at most 2 log V, O(E log E) is the same as O(E log V).

## Parameters in the app

- **New points**: replaces the shared graph with a new random one that has the number of points set by the slider. Every Kruskal and Prim viewport on the sheet uses the same graph and starts over.
- **points slider**: 6 to 24 points. It only takes effect when you press New points.
- **Drag a vertex**: moves it (press within 14 points of it). The edges are recomputed, so the triangulation can change, and every viewport re-records and restarts from step 0. Vertices never end up on top of one another.
- **Edge colors**: unseen edges are thin and light, the edge being considered is orange, accepted edges are thick blue, rejected edges are dashed and grey.
- **Vertex colors**: each vertex takes one of six colors by the position of its tree's root, so trees that merge change color.
- **Next edges list**: when the stage is at least 560 points wide, a column on the right lists the next 10 unseen edges in sorted order with their weights. It is hidden on narrower stages.
- **Transport**: Restart, Step back, Play / Pause, Step, Finish, a speed slider up to 60 steps per second and a scrubber. Space plays or pauses, Left and Right step one operation.
- The header reads the algorithm name, the tree weight so far and the accepted edges out of `n - 1`.

## Numerical notes and pitfalls

- **Union-find without union by size or rank degenerates.** Always linking one root under the other can build a long chain, so a single `find` costs O(V) and the whole run slows toward O(V * E). Union by size (as here) or path compression keeps it nearly constant.
- **Ties.** Equal weights can give several different MSTs, all with the same total weight. The order in which tied edges are tried decides which is reported; the app breaks ties by edge index.
- **Disconnected graphs.** Kruskal then returns a minimum spanning forest, one tree per component, and never reaches `n - 1` edges. The app's graphs are always connected.
- **Float weights.** Lengths are floating point, so compare with a total order (`total_cmp` in Rust) and do not rely on exact equality.
- **Stopping early.** After `n - 1` accepted edges the remaining edges cannot matter; the app stops there, so the list may not be shown to the end.

## Applications

- Network design: laying out cable, roads or pipes at minimum total cost.
- Clustering: **single-linkage** clustering is Kruskal's algorithm stopped early; cutting the MST's longest edges gives the clusters.
- Image segmentation: pixels are vertices and edges are color differences.
- Approximating the traveling salesman problem: a walk around the MST costs at most twice the best tour when distances obey the triangle inequality.

## History

Joseph Kruskal published the algorithm in 1956. The problem is older: Otakar Boruvka first solved it in 1926, designing an electrical network for Moravia, and Vojtech Jarnik gave another method in 1930 (see Prim's algorithm).

## References

- J. B. Kruskal, "On the shortest spanning subtree of a graph and the traveling salesman problem", Proceedings of the American Mathematical Society, 1956.
- O. Boruvka, "O jistem problemu minimalnim", Prace Moravske prirodovedecke spolecnosti, 1926.
- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapter 21, "Minimum Spanning Trees".
- R. Sedgewick and K. Wayne, *Algorithms*, 4th edition, section 4.3.
- Wikipedia, "Kruskal's algorithm".
