# Prim's algorithm

## Overview

A **minimum spanning tree** (MST) of a connected, weighted graph joins every vertex with no cycle and the smallest total weight. **Prim's algorithm** grows it from a single vertex: keep one tree, and at each step add the lightest edge that leaves the tree, together with the vertex at its far end.

Unlike Kruskal's algorithm, which merges many small trees, Prim's always has exactly one tree, and it only ever looks at edges touching that tree.

The app draws 6 to 24 random points (14 at the start) joined by Delaunay edges, and an edge's weight is its length times 100. The run starts at vertex 0. Vertices already in the tree are filled blue; the edge being tested turns orange and then either joins the tree (thick blue) or is rejected (dashed grey) because both of its ends are already in.

## Formula

```math
minimize  sum of w(e) over the tree edges T
|T| = n - 1
grow one tree: add the lightest edge leaving it
cut property: the lightest edge across any cut is in some MST
```

| Symbol | Meaning |
| --- | --- |
| `n` | the number of vertices |
| `w(e)` | the weight of edge `e` (here its length times 100) |
| `T` | the set of accepted edges, the spanning tree |

The **cut property** drives the proof. Split the vertices into "in the tree" and "not yet". The lightest edge across that split belongs to some MST, so adding it can never lead the tree astray.

## Intuition

### One tree, always growing

Picture a blob spreading over the points. At every step it reaches out along the cheapest connection to any vertex still outside. The tree never needs to be merged with anything.

### A priority queue of candidate edges

The candidates are all edges with one end in the tree. A **min-heap** keeps the lightest on top. When a vertex joins, the edges from it to outside vertices are pushed.

### Stale entries

An edge may sit in the heap after its far end has joined the tree through another edge. When it is popped, both ends are in the tree, and it is rejected. The app shows these as dashed grey edges.

## Algorithm

1. Put vertex 0 in the tree and push its edges onto a min-heap keyed by weight.
2. Pop the lightest edge. If both ends are already in the tree, reject it (it is stale).
3. Otherwise accept it, add the new vertex to the tree, and push that vertex's edges that lead outside the tree.
4. Repeat until the heap is empty; the tree then has `n - 1` edges.

```
in_tree = {0}; heap = edges of 0
while heap:
    (w, u, v) = heap.pop_min()
    if u in in_tree and v in in_tree: continue    # stale
    accept (u, v); add the new vertex x to in_tree
    for each edge (x, y) with y not in in_tree: heap.push
```

The app records each step as an operation (`Consider`, `Accept`, `Reject`, `Reach`) and replays the log, so stepping backwards is exact.

## Complexity

| Quantity | Cost |
| --- | --- |
| time with a binary heap | O(E log V) |
| time with an array, no heap | O(V^2), best for dense graphs |
| space | O(V + E) |

With a Fibonacci heap the time improves to O(E + V log V).

## Parameters in the app

- **New points**: replaces the shared graph with a new random one that has the number of points set by the slider. Every Kruskal and Prim viewport on the sheet uses the same graph and starts over.
- **points slider**: 6 to 24 points. It only takes effect when you press New points.
- **Drag a vertex**: moves it (press within 14 points of it). The edges are recomputed, so the triangulation can change, and every viewport re-records and restarts from step 0. Vertices never end up on top of one another.
- **Edge colors**: unseen edges are thin and light, the edge being considered is orange, accepted edges are thick blue, rejected edges are dashed and grey.
- **Candidate edges**: when the stage is at least 560 points wide, a list titled "candidate edges by weight" shows the lightest ten edges still waiting in Prim's heap, which are the unseen edges with at least one end in the tree.
- **Vertex fill**: vertices in the tree are filled blue; the others are hollow. The vertex that joined last gets an extra ring.
- **Transport**: Restart, Step back, Play / Pause, Step, Finish, a speed slider up to 60 steps per second and a scrubber. Space plays or pauses, Left and Right step one operation.
- The header reads the algorithm name, the tree weight so far and the accepted edges out of `n - 1`.

## Numerical notes and pitfalls

- **Needs a connected graph.** On a disconnected graph Prim's grows only the component of the start vertex. The app's graphs are always connected.
- **Stale heap entries.** Without decrease-key, an outside vertex can appear in the heap several times. Check both ends when popping, or the tree gets a cycle.
- **Ties.** Equal weights can give different MSTs with the same total weight. The app breaks ties by edge index, and compares floats with a total order.
- **Same answer as Kruskal.** With distinct weights the MST is unique, so Prim's and Kruskal's report the same tree weight on the same graph; the order of discovery differs.
- **Dense or sparse.** A heap wins on sparse graphs, a plain array of best-known distances on dense ones.

## Applications

- Cable, pipe and road layout at minimum cost.
- Maze generation: **randomized Prim** adds a random frontier edge instead of the lightest.
- Clustering and image segmentation, as with other MST methods.
- The first step of approximation algorithms for the traveling salesman problem.

## History

Vojtech Jarnik described the method in 1930, Robert Prim rediscovered it in 1957, and Edsger Dijkstra found it again in 1959, which is why it is also called the Jarnik-Prim algorithm. Otakar Boruvka had solved the MST problem in 1926.

## References

- R. C. Prim, "Shortest connection networks and some generalizations", Bell System Technical Journal, 1957.
- V. Jarnik, "O jistem problemu minimalnim", Prace Moravske prirodovedecke spolecnosti, 1930.
- E. W. Dijkstra, "A note on two problems in connexion with graphs", Numerische Mathematik, 1959.
- T. H. Cormen, C. E. Leiserson, R. L. Rivest and C. Stein, *Introduction to Algorithms* (CLRS), chapter 21, "Minimum Spanning Trees".
- Wikipedia, "Prim's algorithm".
