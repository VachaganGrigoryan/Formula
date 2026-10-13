# 2D rotation

## Overview

A **rotation** turns every point of the plane about a fixed center by the same angle θ, keeping all lengths and angles unchanged. In coordinates it takes two lines of arithmetic: each new coordinate is a mix of the old `x` and `y`, weighted by `cos θ` and `sin θ`. Written as a 2 × 2 matrix, it is the **rotation matrix** `R(θ)`.

Rotation is one of the three basic rigid motions, together with translation and reflection, and every 2D graphics system implements it: sprite and icon rotation, gauge needles, tilted text, and the `rotate()` transform in CSS and canvas APIs. The app turns a letter F about the origin and, below it, uses the same geometry to decide how far to tilt crowded month labels on a bar chart before they stop overlapping.

## Formula

```math
x' = x cos θ − y sin θ
y' = x sin θ + y cos θ
R(θ) = [ [cos θ, −sin θ], [sin θ, cos θ] ],   p' = R(θ) p
p' = c + R(θ) (p − c)
labels collide  if  s cos a < w  and  s sin a < h
```

| Symbol | Meaning |
| --- | --- |
| `(x, y)` | the point before rotation |
| `(x', y')` | the rotated point |
| `θ` | rotation angle in radians; positive is counter-clockwise with y pointing up |
| `R(θ)` | the rotation matrix, written row by row |
| `c` | center of rotation (the origin unless stated) |
| `s` | horizontal spacing between adjacent axis labels |
| `w`, `h` | width and height of a label's box |
| `a` | tilt angle of the labels |

The third line rotates about an arbitrary center: move `c` to the origin, rotate, and move back. The last line is the collision test the app uses for tilted labels.

## Intuition

### Columns are rotated axes

Any point is a combination of the unit vectors: `p = x·(1, 0) + y·(0, 1)`. Rotation is linear, so it is enough to know where the two unit vectors go. The x-axis vector lands on `(cos θ, sin θ)` and the y-axis vector on `(−sin θ, cos θ)`, a quarter turn ahead of it. The rotated point is `x` times the first plus `y` times the second, which is exactly the formula. These two vectors are the columns of `R(θ)`, drawn in the app as the colored arrows.

### Derivation from polar coordinates

Write the point as `x = r cos φ`, `y = r sin φ`. Rotating adds θ to the angle and keeps the radius: `x' = r cos(φ + θ)`, `y' = r sin(φ + θ)`. Expanding with the angle-addition identities gives `x' = r cos φ cos θ − r sin φ sin θ = x cos θ − y sin θ`, and similarly for `y'`. In complex numbers this is simply multiplication: `(x + iy) · e^(iθ)`.

### Properties

- `det R = cos²θ + sin²θ = 1`: areas are preserved and orientation is kept.
- The inverse is the transpose, `R(−θ)`: flip the sign of the sines.
- Rotations compose by adding angles: `R(α) R(β) = R(α + β)`.

### Tilting labels

Two neighboring labels are congruent boxes `s` apart horizontally, both tilted by `a`. In the labels' own rotated frame their offset is `(s cos a, s sin a)`. Two parallel congruent boxes overlap exactly when that offset is shorter than the box along both of its axes, which gives the test above. Flat labels collide when `s < w`; tilting by any `a ≥ asin(h / s)` separates them.

## Algorithm

1. Compute `c = cos θ` and `s = sin θ` once (many libraries offer `sin_cos`).
2. For each point, subtract the center if it is not the origin.
3. Set `x' = x·c − y·s` and `y' = x·s + y·c`, using the *old* `x` for both lines.
4. Add the center back.

```
(s, c) = sin_cos(theta)
for p in points:
    dx, dy = p.x - cx, p.y - cy
    p.x = cx + dx*c - dy*s
    p.y = cy + dx*s + dy*c
```

## Complexity

| Case | Time | Space |
| --- | --- | --- |
| one point | O(1): 4 multiplications, 2 additions | O(1) |
| n points, same angle | O(n), one `sin`/`cos` pair in total | O(1) extra |
| composing two rotations | O(1) | O(1) |
| label collision test | O(1) per neighboring pair | O(1) |

## Parameters in the app

- **θ** (−180° to 180°): the rotation angle; moving the slider stops the spin.
- **Spin / Stop**: animates θ at about 0.5 radians per second.
- **Drag P**: moves the point being rotated (kept within radius 1.1); the dashed arc shows the path it travels to `P'`, and the readout shows both lines of the formula with numbers.
- **Label tilt** (0° to 90°): tilts the month labels of the bar chart. Labels that collide with a neighbor are highlighted, and a note counts the colliding pairs.

## Numerical notes and pitfalls

- **Screen coordinates point down.** With y growing downward, the same formula turns clockwise on screen. Either flip the sign of θ or flip y when converting to screen coordinates, as the app does.
- **Degrees versus radians.** Trigonometric functions take radians; convert with `θ·π/180`.
- **Update in place.** Writing `x = x·c − y·s` and then `y = x·s + y·c` uses the new `x` in the second line. Keep the old value in a temporary.
- **Accumulated drift.** Multiplying a matrix by a small rotation every frame slowly lets rounding errors make it non-orthogonal, so shapes shear and grow. Store the angle and rebuild the matrix, or re-orthonormalize it periodically.
- **Center of rotation.** Rotating about the origin when you meant the shape's center swings the shape around the corner of the screen. Translate, rotate, translate back.
- **Rotating images.** Rotating pixel positions forward leaves holes. Loop over destination pixels instead and rotate them backward by `R(−θ)` to sample the source.
- **Text readability.** Labels tilted past 90° read upside down; tilts between 30° and 60° are a common compromise.

## Applications

- Tilting crowded axis labels on charts, and orienting text along radar-chart spokes.
- Compass needles, gauges, dials and spinning loading indicators.
- Sprite, icon and camera rotation in 2D games and UI frameworks.
- Image rotation in editors and document scanners.
- Robot and vehicle poses in planar motion planning.
- Building block of 3D rotations: a 3 × 3 rotation about each coordinate axis embeds this 2 × 2 block.

## History

The formula is a restatement of the angle-addition identities, which Ptolemy effectively used in his chord tables in the *Almagest* (2nd century). Leonhard Euler's formula `e^(iθ) = cos θ + i sin θ` (1748) showed that rotation is multiplication by a unit complex number, and Euler also described general 3D rotations by three angles. The word *matrix* was coined by James Joseph Sylvester in 1850, and Arthur Cayley's 1858 memoir set out the algebra of matrices in which `R(θ)` is the standard example of an orthogonal matrix.

## References

- Wikipedia, "Rotation matrix".
- G. Strang, *Introduction to Linear Algebra*, chapter on orthogonal matrices.
- J. D. Foley, A. van Dam, S. K. Feiner and J. F. Hughes, *Computer Graphics: Principles and Practice*, chapter on geometric transformations.
- T. Needham, *Visual Complex Analysis*, chapter 1.
- A. Cayley, "A Memoir on the Theory of Matrices", Philosophical Transactions of the Royal Society, 1858.
