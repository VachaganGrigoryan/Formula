# Euclidean distance

## Overview

The **Euclidean distance** is the ordinary straight-line length between two points, the distance a ruler measures. In two dimensions it is the Pythagorean theorem applied to the horizontal and vertical gaps between the points; in any number of dimensions it is the square root of the sum of squared coordinate differences.

It is the default notion of "closeness" in geometry, graphics and data analysis. Every "which item is nearest?" question, from a scatter-plot tooltip snapping to the point under the cursor to a nearest-neighbor classifier, starts with it. It also drives **K-means clustering**, which repeatedly assigns each point to its nearest center. The app shows both uses.

## Formula

```math
d = √( (x_{2} − x_{1})^{2} + (y_{2} − y_{1})^{2} )
d(p, q) = √( Σ_{i=1}^{n} (q_{i} − p_{i})^{2} )
J = Σ_{j} |x_{j} − μ_{c(j)}|^{2}
```

| Symbol | Meaning |
| --- | --- |
| `(x₁, y₁)`, `(x₂, y₂)` | the two points in the plane |
| `p`, `q` | points with `n` coordinates `p_i`, `q_i` |
| `d` | the distance, always ≥ 0 |
| `μ_c` | the centroid (mean) of cluster `c` |
| `c(j)` | the cluster that point `x_j` is assigned to |
| `J` | the K-means objective: total squared distance to the assigned centroids |

The distance is symmetric, `d(p, q) = d(q, p)`, is zero only when the points coincide, and satisfies the triangle inequality `d(p, r) ≤ d(p, q) + d(q, r)`. K-means minimizes `J`, the sum of squared Euclidean distances.

## Intuition

Draw the segment between the two points and drop a horizontal and a vertical leg from its ends. The legs have lengths `Δx = |x₂ − x₁|` and `Δy = |y₂ − y₁|` and meet at a right angle, so the segment is the hypotenuse: `d² = Δx² + Δy²`. In three dimensions the same step is applied twice (first across the floor, then up), which is why the general formula simply adds one squared term per coordinate.

All points at distance `d` from a center form a circle (a sphere in 3D). That gives a picture of "nearest": grow a circle around the query point until it touches the first data point. Nothing lies inside that circle; in the app it is drawn shaded around the mouse.

Because the square root is increasing, comparing `d²` gives the same ordering as comparing `d`. Nearest-neighbor searches therefore skip the root entirely and take it only for the one distance they report.

### Why K-means uses squared distance

For a fixed set of points, the location `m` that minimizes `Σ |x_j − m|²` is their mean: setting the derivative `−2 Σ (x_j − m)` to zero gives `m = mean(x)`. So each K-means round makes two moves that can only lower `J`: reassigning a point to a closer centroid, and moving a centroid to the mean of its members.

## Algorithm

Nearest point (linear scan):

1. Set `best = none`, `bestD2 = ∞`.
2. For each point `p`, compute `d2 = (p.x − q.x)² + (p.y − q.y)²`.
3. If `d2 < bestD2`, remember `p` and `d2`.
4. Return the best point and `√bestD2`.

K-means (Lloyd's algorithm):

1. Choose `k` initial centroids (random points, or k-means++).
2. **Assign**: put every point in the cluster of its nearest centroid.
3. **Update**: move each centroid to the mean of its points; an empty cluster keeps its centroid.
4. Repeat 2 and 3 until no centroid moves.

```
repeat
    for each point x:  c(x) = argmin_j |x - mu_j|^2
    for each cluster j: mu_j = mean of { x : c(x) = j }
until no mu_j changed
```

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| one distance in n dimensions | O(n) | O(1) |
| nearest of N points, linear scan | O(N·n) | O(1) |
| nearest with a k-d tree (low n) | O(log N) average after O(N log N) build | O(N) |
| one K-means iteration, k clusters | O(N·k·n) | O(N + k·n) |

K-means always terminates, because `J` never increases and there are finitely many assignments. In practice it needs tens of iterations, though contrived inputs can need exponentially many.

## Parameters in the app

- **Nearest point / K-means**: switches between the two demonstrations.
- **Mouse**: in *Nearest point* mode, the query point; the tooltip shows the nearest point's index, coordinates and `d`, and the colored legs show `Δx` and `Δy`.
- **Clusters k** (2 to 6): the number of centroids; changing it restarts K-means.
- **Run / Pause**: steps K-means automatically, one round every 0.7 s.
- **Step**: performs one assign-and-update round by hand (disabled once converged).
- **New centroids**: restarts from fresh random centroids on the same data.
- **New data**: draws a new set of 90 points in four jittered blobs.

## Numerical notes and pitfalls

- **Skip the root when comparing.** `d²` orders points exactly like `d` and avoids a square root per candidate.
- **Overflow and underflow.** Squaring very large or very small coordinates can overflow to infinity or underflow to zero. `hypot(dx, dy)` rescales internally and is safe; use it when magnitudes are extreme.
- **Equal axis scaling.** On screen a distance is only Euclidean if both axes use the same pixels per unit. The app draws its 100 by 60 data space with equal scale; a stretched chart makes the visually nearest point differ from the mathematically nearest one.
- **Feature units.** In data analysis, a feature measured in thousands dominates one measured in fractions. Standardize features (subtract the mean, divide by the standard deviation) before using Euclidean distance or K-means.
- **High dimensions.** As `n` grows, distances between random points concentrate around one value, so "nearest" means less.
- **K-means local minima.** The result depends on the starting centroids: run several restarts and keep the lowest `J`, or seed with k-means++. K-means favors round, similar-sized clusters.

## Applications

- Snapping chart tooltips and hover highlights to the nearest data point.
- Hit-testing: did the click land within `r` pixels of a handle or node?
- Collision and overlap checks between circles in games and network layouts.
- Nearest-neighbor classification and recommendation (k-NN).
- K-means for customer segmentation and color quantization of images.

## History

The theorem behind the formula is attributed to Pythagoras (6th century BC) and proved in Euclid's *Elements* (around 300 BC), Book I, Proposition 47. Writing it as a formula in coordinates became possible with the analytic geometry of René Descartes and Pierre de Fermat in the 17th century. The adjective *Euclidean* distinguishes this flat-space distance from the non-Euclidean geometries of the 19th century, and Hermann Minkowski's family of `L_p` distances places it at `p = 2`.

K-means was proposed independently several times: Hugo Steinhaus described the idea in 1956, Stuart Lloyd developed the standard iteration at Bell Labs in 1957 (published in 1982), and James MacQueen coined the name "k-means" in 1967. The k-means++ seeding rule is due to Arthur and Vassilvitskii (2007).

## References

- Euclid, *Elements*, Book I, Proposition 47.
- Wikipedia, "Euclidean distance" and "k-means clustering".
- S. P. Lloyd, "Least squares quantization in PCM", IEEE Transactions on Information Theory, 1982.
- J. MacQueen, "Some methods for classification and analysis of multivariate observations", Proceedings of the Fifth Berkeley Symposium, 1967.
- D. Arthur and S. Vassilvitskii, "k-means++: the advantages of careful seeding", SODA 2007.
- T. Hastie, R. Tibshirani and J. Friedman, *The Elements of Statistical Learning*, chapter 14.
