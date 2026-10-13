# Voronoi diagram

## Overview

Scatter a set of points, called **seeds** or sites, over the plane. The **Voronoi diagram** gives each seed a **cell**: the region of all locations that are closer to that seed than to any other. The cells tile the plane without gaps or overlaps, every cell is a convex polygon, and each border between two cells is a piece of the perpendicular bisector of their seeds.

It answers "which one is nearest?" for every location at once, so it appears wherever influence spreads evenly from sources: the catchment area of a hospital, the coverage of a cell tower. The same pattern grows in giraffe skin, dried mud and insect wings. The app computes every cell exactly, lets the seeds drift, and can relax them into an even honeycomb with Lloyd's algorithm.

## Formula

```math
V(p_{i}) = { x | d(x, p_{i}) ≤ d(x, p_{j}) for all j ≠ i }
H_{ij} = { x | (x − m_{ij}) · (p_{j} − p_{i}) ≤ 0 },   m_{ij} = (p_{i} + p_{j}) / 2
V(p_{i}) = B cut by every H_{ij}, j ≠ i
p_{i} := centroid(V(p_{i}))   (one step of Lloyd's algorithm)
```

| Symbol | Meaning |
| --- | --- |
| `p_i` | the i-th seed |
| `V(p_i)` | the Voronoi cell of seed `p_i` |
| `d(x, p)` | Euclidean distance from location `x` to `p` |
| `m_ij` | midpoint between seeds `p_i` and `p_j` |
| `H_ij` | half-plane of locations at least as close to `p_i` as to `p_j` |
| `B` | the bounding box (the stage), which closes off the outer cells |

The bisector of `p_i` and `p_j` is the line through their midpoint perpendicular to the segment joining them. A cell is the intersection of the half-planes on its seed's side of all these bisectors, cut down to the bounding box.

## Intuition

Picture a circle growing from every seed at the same speed, like ripples on a pond. Where two ripples meet they stop, and the meeting points of two equal-speed ripples lie on the perpendicular bisector. The places where three ripples meet are the Voronoi **vertices**: each is equidistant from three seeds, so it is the center of a circle through them that contains no other seed.

Because each half-plane is convex and an intersection of convex sets is convex, every cell is a convex polygon. That is what makes exact computation simple: start from the whole box and cut away, one bisector at a time, the part that is closer to some other seed.

### The dual: Delaunay triangulation

Join two seeds with a line whenever their cells share an edge. The result is the **Delaunay triangulation**, the triangulation that avoids thin slivers by maximizing its smallest angle. The two structures carry the same information, and building either one yields the other.

### Lloyd relaxation

Randomly scattered seeds give cells of very uneven size. Moving each seed to the centroid of its cell and recomputing the cells, again and again, spreads the seeds out until each one sits at the center of mass of its own cell: a **centroidal Voronoi tessellation**, whose cells look like a honeycomb of near-regular hexagons.

## Algorithm

Exact cells by half-plane clipping, as in the app:

1. For seed `p_i`, start with the bounding box as a 4-corner polygon.
2. For every other seed `p_j` (skipping duplicates of `p_i`), compute the midpoint `m` and the direction `n = p_j − p_i`.
3. Clip the polygon to `(x − m) · n ≤ 0` with one pass of Sutherland–Hodgman: keep corners on the inner side, and where an edge crosses the line, insert the crossing point.
4. The polygon that remains is the cell. Repeat for each seed.

```
clip(poly, side):
    out = []
    for each edge (a, b) of poly:
        if side(a) <= 0: out.push(a)
        if (side(a) <= 0) != (side(b) <= 0):
            t = side(a) / (side(a) - side(b))
            out.push(a + t * (b - a))
    return out
```

Lloyd's algorithm: compute all cells, move each seed to its cell's centroid (shoelace formula), and repeat until the seeds stop moving. The app moves seeds part of the way each frame.

## Complexity

| Method | Time | Space |
| --- | --- | --- |
| half-plane clipping, all cells | O(n² · k), k = average cell size (about 6) | O(n) |
| per-pixel nearest seed, W × H image | O(W · H · n) | O(W · H) |
| Fortune's sweep line | O(n log n) | O(n) |
| via Delaunay (Bowyer–Watson, random order) | O(n log n) expected | O(n) |

A 2D Voronoi diagram of n seeds has at most 2n − 5 vertices and 3n − 6 edges, so cells average fewer than 6 edges. Clipping is quadratic but simple, and fast for a few hundred seeds.

## Parameters in the app

- **Seeds** (2 to 150) and **Scatter**: how many seeds, and a fresh random layout.
- **Style**: *Colorful*, *Giraffe*, *Cracked mud* and *Dragonfly wing*; the natural styles shrink cells by a small gap to imitate the pattern.
- **Drift**: seeds wander slowly and bounce off the borders.
- **Relax (Lloyd)**: seeds slide toward their cell centroids, evening out the cells.
- **Delaunay edges**: joins seeds whose cells share an edge.
- **Seeds**: shows or hides the seed dots.
- Mouse: drag a seed to move it, click empty space to add one, right-click a seed to remove it. Hovering draws the distances to the nearest seed and the runner-up.

## Numerical notes and pitfalls

- **Duplicate seeds.** Two seeds at the same spot have no bisector (the direction is zero). Skip them, or remove duplicates first.
- **Tolerance at the clip line.** Use `≤ 0` consistently so that corners lying exactly on a bisector are kept once, not dropped or duplicated.
- **Unbounded cells.** Cells of seeds on the convex hull extend to infinity. Clipping starts from a bounding box, which closes them off; Fortune-style algorithms must add the box explicitly.
- **Degenerate vertices.** Four or more cocircular seeds (a square grid) meet at one vertex; robust libraries use exact predicates.
- **Pixel versus exact.** A per-pixel nearest-seed render is easy and is how GPU shaders and Worley noise do it, but its edges alias and it gives no polygons for areas or centroids.
- **Lloyd convergence.** Each step lowers the total squared distance from points to their seeds, but convergence is slow near the end; stop when seeds move less than a tolerance.

## Applications

- Nearest-facility maps: closest hospital, school, store or cell tower.
- Natural-neighbor interpolation of scattered measurements, and Thiessen polygons for rainfall.
- Procedural textures: stone, scales, cracked earth, cells (Worley noise).
- Mesh generation and stippling via centroidal Voronoi tessellations.
- Robot path planning along Voronoi edges, which keep maximum clearance from obstacles.
- Crystallography (the Wigner–Seitz cell) and the 1854 cholera map of John Snow.

## History

René Descartes sketched a similar partition of space in 1644. Peter Gustav Lejeune Dirichlet used the construction in 1850 for quadratic forms, and Georgy Voronoi generalized it to any dimension in 1908, which is why the cells are also called Dirichlet domains. The meteorologist Alfred Thiessen rediscovered them in 1911 to average rainfall between gauges (Thiessen polygons), and Boris Delaunay, a student of Voronoi, introduced the dual triangulation in 1934. Steven Fortune's O(n log n) sweep-line algorithm appeared in 1986, and Stuart Lloyd's relaxation dates from 1957.

## References

- A. Okabe, B. Boots, K. Sugihara and S. N. Chiu, *Spatial Tessellations: Concepts and Applications of Voronoi Diagrams*, Wiley.
- M. de Berg, O. Cheong, M. van Kreveld and M. Overmars, *Computational Geometry: Algorithms and Applications*, chapter 7.
- F. Aurenhammer, "Voronoi diagrams: a survey of a fundamental geometric data structure", ACM Computing Surveys, 1991.
- S. Fortune, "A sweepline algorithm for Voronoi diagrams", Algorithmica, 1987.
- Q. Du, V. Faber and M. Gunzburger, "Centroidal Voronoi tessellations: applications and algorithms", SIAM Review, 1999.
- Wikipedia, "Voronoi diagram" and "Lloyd's algorithm".
