# Matrix transformation

## Overview

A 2 × 2 **matrix** describes a *linear transformation* of the plane: a rule that moves every point while keeping the origin fixed, straight lines straight, and evenly spaced grid lines evenly spaced. Rotations, scalings, shears, reflections and projections are all of this kind, and so is any combination of them.

The whole transformation is pinned down by just two vectors: where the basis vectors `i = (1, 0)` and `j = (0, 1)` land. Those two images are the columns of the matrix. The app draws the original grid faintly and the transformed grid on top, so you can drag the tips of `i` and `j` and watch the entire plane follow.

Two numbers summarize what a matrix does. The **determinant** is the factor by which it scales areas, negative when it flips the plane over. The **eigenvectors** are the special directions that the matrix only stretches, never turns; the stretch factors are the **eigenvalues**.

## Formula

```math
x' = a x + b y
y' = c x + d y
A = [ [a, b], [c, d] ],   p' = A p
det A = ad − bc
A v = λ v,   λ² − (a + d) λ + (ad − bc) = 0
λ = (a + d)/2 ± √( ((a + d)/2)² − (ad − bc) )
```

| Symbol | Meaning |
| --- | --- |
| `(x, y)` | a point before the transformation |
| `(x', y')` | the same point after it |
| `a, b, c, d` | the entries of `A`, row by row; the columns `(a, c)` and `(b, d)` are the images of `i` and `j` |
| `det A` | signed area of the image of the unit square |
| `λ` | an eigenvalue: the stretch factor along an eigenvector |
| `v` | an eigenvector: a nonzero direction with `A v = λ v` |

The quadratic in the fifth line is the **characteristic equation**. Its two roots are the eigenvalues; when the number under the square root is negative they are complex and no real direction is left unturned.

## Intuition

### Columns are where the basis lands

Every point is a combination of the basis vectors: `p = x·i + y·j`. A linear map respects combinations, so `A p = x·(A i) + y·(A j)`. Knowing the two images `A i = (a, c)` and `A j = (b, d)` is therefore enough to move every point, and writing them side by side as columns gives the matrix. Dragging an arrow tip in the app edits one column.

### The determinant is an area

The unit square spanned by `i` and `j` maps to the parallelogram spanned by the two columns. Its signed area is `ad − bc`, and every other region is scaled by the same factor. A negative determinant means the parallelogram is "inside out": the plane has been mirrored, which the app shows by tinting the square and by the letter F reading backwards. A zero determinant means the columns lie on one line, so the whole plane is squashed onto that line (or onto the origin).

### Eigenvectors stay on their line

Most vectors change direction under `A`. An eigenvector does not: `A v` lands on the same line through the origin, only scaled by `λ`. Along that line the transformation is just a stretch, so eigenvectors are the natural axes of the matrix. A shear has only one such line; a rotation has none (its eigenvalues are complex); a uniform scaling has every direction as an eigenvector.

### Composition is multiplication

Doing `B` first and then `A` is the single matrix `A B`. The order matters: rotating and then shearing is not the same as shearing and then rotating.

## Algorithm

1. To transform a point, compute `x' = a x + b y` and `y' = c x + d y`. Straight lines map to straight lines, so a grid line needs only its two endpoints transformed.
2. The determinant is `ad − bc`.
3. For the eigenvalues, let `h = (a + d)/2` and `D = h² − det`. If `D < 0` the eigenvalues are `h ± i √(−D)`; otherwise they are `h ± √D`.
4. For each real eigenvalue `λ`, an eigenvector is perpendicular to a nonzero row of `A − λI`: take `(b, λ − a)` or `(λ − d, c)`, whichever is longer, and normalize it. If both are zero, `A = λI` and every direction is an eigenvector.

```
h = (a + d) / 2
D = h*h - (a*d - b*c)
if D < -eps:  complex pair h +- i*sqrt(-D)
else:
    for lam in (h + sqrt(D), h - sqrt(D)):
        u = (b, lam - a);  w = (lam - d, c)
        v = longer of u, w;  normalize v unless it is ~0
```

The app animates a matrix by interpolating from the identity, `I + t (A − I)` for `t` from 0 to 1, so you can watch the grid move into place.

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| transform one point | O(1): 4 multiplications, 2 additions | O(1) |
| transform n points | O(n) | O(1) extra |
| determinant | O(1) | O(1) |
| eigenvalues and eigenvectors (2 × 2, closed form) | O(1), one square root | O(1) |
| compose two matrices | O(1): 8 multiplications | O(1) |

For n × n matrices the same questions cost O(n²) per point, O(n³) for a determinant by elimination, and an iterative O(n³) algorithm (such as QR) for eigenvalues.

## Parameters in the app

- **a, b, c, d** (−5 to 5): the matrix entries. The first column `(a, c)` is where `i` lands, the second `(b, d)` where `j` lands.
- **Drag the tips of i and j**: edits the columns directly; with **Snap tips to 0.5** the tips land on half-unit points.
- **Presets**: identity, scale, rotate 30°, shear, reflect, project and a singular matrix. Choosing one plays the morph from the identity.
- **Animate**: replays the morph from the identity to the current matrix.
- **House / Letter F / Unit square**: the figure drawn before (dashed) and after (solid) the transformation. The F is asymmetric, so reflections are easy to spot.
- The readout shows the matrix, `det` with what it means for areas and orientation, and the eigenvalues; dashed lines through the origin are the eigenvectors, labeled with their `λ`.

## Numerical notes and pitfalls

- **Row versus column vectors.** This page multiplies column vectors on the right, `p' = A p`, so the columns are the images of the basis. Some graphics APIs use row vectors, `p' = p A`, which transposes everything; mixing the two conventions silently swaps `b` and `c`.
- **Order of composition.** `A B` applies `B` first. Writing transforms in the order you think of them ("rotate, then scale") gives the product in the opposite order.
- **Near-singular matrices.** When `det` is close to zero the inverse has huge entries and solving `A x = y` amplifies small errors. Compare `|det|` with the size of the entries, not with zero.
- **Repeated eigenvalues.** When the discriminant is near zero, rounding can turn one eigenvalue into two very close ones with nearly parallel eigenvectors. Treat a small `|D|` as a repeated root, as the app does.
- **Choosing the eigenvector formula.** Using only `(b, λ − a)` fails when that row of `A − λI` is zero, for example for a diagonal matrix. Picking the longer of the two candidates avoids dividing by zero.
- **Screen coordinates point down.** With y growing downward a positive rotation turns clockwise on screen; flip y when drawing, as the app does.
- **Complex eigenvalues are not an error.** They mean the matrix rotates every direction; `|λ|` is still the scaling per step, which matters for stability.

## Applications

- 2D graphics: every scale, rotation, shear and mirror of sprites, text and images, combined into one matrix per object.
- Image warping and texture mapping, where the matrix maps destination pixels back to the source.
- Principal component analysis: the eigenvectors of a covariance matrix are the directions of greatest spread in data.
- Markov chains and population models, whose long-run behavior is the eigenvector of the largest eigenvalue.
- Stability of linear dynamical systems and of numerical schemes: a system decays when every `|λ| < 1`.
- Mechanics: principal axes of stress and of inertia are eigenvectors.

## History

Determinants came before matrices: Seki Takakazu (1683) and Gottfried Leibniz (1693) used them independently to solve systems of equations, and Gabriel Cramer published his rule in 1750. Leonhard Euler and Joseph-Louis Lagrange met eigenvalue problems in rigid-body rotation and in the oscillations of mechanical systems, and Augustin-Louis Cauchy proved in 1829 that symmetric matrices have real eigenvalues. James Joseph Sylvester coined the word *matrix* in 1850, and Arthur Cayley's 1858 memoir defined matrix multiplication as the composition of linear substitutions. David Hilbert's work on integral equations around 1904 popularized the prefix *eigen* ("own", "characteristic") that gives eigenvalues their name.

## References

- G. Strang, *Introduction to Linear Algebra*, chapters on linear transformations, determinants and eigenvalues.
- G. Sanderson (3Blue1Brown), *Essence of Linear Algebra*, video series, 2016.
- S. Axler, *Linear Algebra Done Right*, chapter 5.
- L. N. Trefethen and D. Bau, *Numerical Linear Algebra*, SIAM, 1997.
- A. Cayley, "A Memoir on the Theory of Matrices", Philosophical Transactions of the Royal Society, 1858.
- Wikipedia, "Transformation matrix" and "Eigenvalues and eigenvectors".
