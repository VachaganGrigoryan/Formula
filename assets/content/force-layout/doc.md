# Springs and repulsion

## Overview

A force-directed layout draws a network by pretending it is a physical system. Every edge is a spring that wants a certain length, and every pair of nodes repels like two equal electric charges. Starting from random positions, the simulation lets these forces push and pull the nodes until they come to rest. The resting shape is usually a readable drawing: connected nodes sit close together, unrelated ones spread apart, and clusters emerge on their own without anyone placing them.

The method needs nothing but the graph itself, which is why it is the default layout for social networks, knowledge graphs, dependency diagrams and molecule viewers. Dragging a node and letting the rest rebalance is also one of the most satisfying interactions in data visualization.

## Formula

```math
F_{s} = −k (d − x_{0})
F_{r} = k_{r} / d^{2}
F_{i} = Σ_{edges} F_{s} · u_{ij} + Σ_{j ≠ i} F_{r} · u_{ij} − g · p_{i}
v_{i} = (v_{i} + F_{i}) · damping,   p_{i} = p_{i} + v_{i}
```

| Symbol | Meaning |
| --- | --- |
| `d` | distance between the two nodes |
| `k` | spring stiffness (Hooke's constant) |
| `x0` | spring rest length |
| `kr` | repulsion strength (the product of the two "charges") |
| `u_ij` | unit vector from node j to node i |
| `g` | weak gravity pulling every node toward the center |
| `p_i`, `v_i` | position and velocity of node i |
| `damping` | fraction of velocity kept each step (0.85 in the app) |

A positive force pushes the pair apart along `u_ij`; a negative one pulls them together. A spring stretched beyond `x0` pulls; one squeezed shorter pushes.

## Intuition

The two forces fight at different ranges. Repulsion falls off as 1 / d², so it is huge for nodes that overlap and negligible for nodes far apart: it acts like personal space. Springs grow linearly with how far they are stretched, so they dominate at long range and reel distant neighbors back in. The equilibrium of a single linked pair is where the two cancel, k (d − x0) = kr / d², which is always a little longer than `x0`.

The whole layout minimizes a total energy: the spring energy ½ k (d − x0)² summed over edges plus the repulsion energy `kr / d` summed over all pairs. Each step moves nodes downhill along the negative gradient of this energy. Damping removes kinetic energy every step, which is what lets the system settle instead of oscillating forever, just as friction stops a real spring.

The extra weak gravity toward the center keeps disconnected pieces from drifting off to infinity, because repulsion between components never stops pushing them apart.

## Algorithm

1. Place the nodes at random positions with zero velocity.
2. For each edge, compute the spring force and add it to both endpoints with opposite signs.
3. For each pair of nodes, compute the repulsion and add it to both, with opposite signs.
4. Add the gravity term −g · p to each node.
5. Update velocity v = (v + F) · damping, cap its length so stiff springs cannot fling a node away, then move `p = p + v`. A dragged node is pinned: its velocity is set to 0.
6. Sum the kinetic energy ½ |v|²; repeat from step 2 until it is close to 0.

```
for each step:
    F = zeros
    for (i, j) in edges:
        f = unit(p[i] - p[j]) * (-k * (d - rest))
        F[i] += f;  F[j] -= f
    for i < j:
        f = unit(p[i] - p[j]) * kr / max(d, 4)^2
        F[i] += f;  F[j] -= f
    for i: v[i] = (v[i] + F[i] - g * p[i]) * damping;  p[i] += v[i]
```

## Complexity

| Part | Time per step | Space |
| --- | --- | --- |
| springs | O(E) | O(V + E) |
| repulsion, all pairs | O(V²) | O(V) |
| repulsion, Barnes-Hut quadtree | O(V log V) | O(V) |
| total until settled | O(I · V²), I iterations (hundreds) | O(V + E) |

The all-pairs repulsion dominates. Up to a few hundred nodes it runs in real time; beyond that, layouts approximate distant groups of nodes by their center of mass (Barnes-Hut) or use multilevel coarsening.

## Parameters in the app

- **Stiffness k**: how hard each spring pulls toward its rest length. Low values give loose, floppy graphs; high values snap edges to length.
- **Rest length x₀**: the length every spring would like to have; scales the whole drawing.
- **Repulsion kr**: how strongly every pair of nodes pushes apart; raise it to spread clusters out.
- **Run / Pause**: start or stop the simulation.
- **Shuffle**: scatter the nodes to new random positions and let them settle again.
- **New graph**: generate four new clusters, each a random tree with a few extra edges, chained together.
- **Show forces**: draw the spring force and the repulsion force on the hovered or dragged node.
- Edge color shows strain: stretched springs in one color, squeezed springs in another, thicker the further from rest.
- Drag a node to pin it and watch the graph rebalance; the kinetic energy readout shows when it has settled.

## Numerical notes and pitfalls

- **Coincident nodes.** Repulsion at `d = 0` divides by zero and has no direction. Clamp the distance to a small minimum (the app uses 4 px) and give overlapping nodes a tiny random nudge.
- **Explosions.** A large time step or stiff spring makes explicit integration overshoot and blow up. Cap the per-step displacement, lower `k`, or use a smaller step.
- **Never settling.** Without damping the energy just sloshes around. Damping below 1, or a "temperature" that cools over time (simulated annealing), guarantees convergence.
- **Local minima.** The layout finds *a* low-energy arrangement, not the best one; twisted or folded drawings happen. Restart from new positions, or start from a spectral or multidimensional-scaling layout.
- **Disconnected graphs.** Components repel each other forever; add gravity toward the center or lay out components separately and pack them.
- **Units.** Spring force grows with pixels and repulsion shrinks with pixels squared, so `k`, `x0` and `kr` must be retuned together if the canvas scale changes.

## Applications

- Network and graph visualization tools such as Gephi, Graphviz's `neato` and `fdp`, Cytoscape and the `d3-force` module.
- Social network maps, citation graphs and knowledge graphs.
- Software dependency and call graphs, and microservice maps.
- Label placement: labels repel each other and are tied to their anchors by springs.
- Biology: protein interaction networks and simple molecular layouts.
- Game and UI physics for floating, self-arranging elements such as bubble charts.

## History

Peter Eades proposed the spring embedder in 1984, using logarithmic springs and inverse-square repulsion between non-adjacent nodes. Thomas Fruchterman and Edward Reingold refined it in 1991 with attractive force d² / k, repulsive force k² / d, and a cooling temperature that limits each move. Tomihisa Kamada and Satoru Kawai (1989) instead made spring lengths equal to graph-theoretic distances and minimized the resulting energy. The name "force-directed" reflects that the drawing is directed purely by simulated forces; the springs and charges come straight from Hooke's law (Robert Hooke, 1678) and Coulomb's law (Charles-Augustin de Coulomb, 1785).

## References

- Peter Eades, "A heuristic for graph drawing", Congressus Numerantium 42, 1984.
- Thomas M. J. Fruchterman and Edward M. Reingold, "Graph drawing by force-directed placement", Software: Practice and Experience, 1991.
- Tomihisa Kamada and Satoru Kawai, "An algorithm for drawing general undirected graphs", Information Processing Letters, 1989.
- Josh Barnes and Piet Hut, "A hierarchical O(N log N) force-calculation algorithm", Nature, 1986.
- Wikipedia, "Force-directed graph drawing".
