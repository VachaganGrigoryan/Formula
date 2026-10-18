# Pythagorean theorem

## Overview

In a right triangle the square on the hypotenuse equals the sum of the squares on the other two sides: a² + b² = c². The tool shows two classic proofs. The first rearranges four copies of the triangle inside a square of side a + b; the second drops the altitude from the right angle and finds two smaller copies of the triangle inside it.

Both proofs are in the Derive tab, step by step, and "Plug in current values" computes c from the current a and b.

## Formula

```math
a² + b² = c²
c = √(a² + b²)
```

| Symbol | Meaning |
| --- | --- |
| a, b | the legs: the two sides that meet at the right angle |
| c | the hypotenuse: the side opposite the right angle |
| p, q | the parts of c on either side of the altitude's foot (similar-triangles proof) |
| h | the altitude from the right angle to the hypotenuse |

## Intuition

**Rearrangement.** Put four copies of the triangle in the corners of a square of side a + b, with their right angles at the corners. Their hypotenuses enclose a tilted square of side c. Now slide three of the triangles so that they pair up into two a × b rectangles: the uncovered part becomes a square of side a and a square of side b. Nothing was added or removed, so the uncovered area is the same: c² = a² + b².

**Similar triangles.** The altitude from the right angle cuts the triangle into two smaller right triangles, and each has the same angles as the whole one. Scaled copies have proportional sides, which gives a² = c·p and b² = c·q. Adding them, a² + b² = c(p + q) = c².

## Algorithm

The rearrangement animation places the triangles at the corners, then moves three of them by translations only: (0, a), (−b, 0) and (a, −b) in a square with corners (0, 0) and (a + b, a + b). Each moves in its own third of the timeline with a smoothstep ease, so the motion is rigid at every moment.

The similar-triangles view puts the hypotenuse on the x axis and the right-angle corner at (p, h) with p = a²/c and h = ab/c, so the legs come out with lengths exactly a and b.

## Complexity

Constant: four triangles of three points each, and one square root for c.

## Parameters in the app

- **Rearrangement / Similar triangles**: which proof the stage shows.
- **a, b**: the legs, from 0.5 to 6.
- **t** (rearrangement): the position in the animation, 0 for the tilted c² square and 1 for the a² and b² squares. Moving it pauses the animation; *Play* continues from there.
- **Stage**: the uncovered area is tinted; the legs are drawn in two colors so you can follow each triangle. In the similar-triangles view the two smaller triangles are shaded and p, q and the products c·p and c·q are shown.

## Numerical notes and pitfalls

- Computing c = √(a² + b²) directly overflows when a or b is near the largest representable number and loses precision when they are tiny. Library functions such as `hypot` scale first; the app uses `f64::hypot` for the drawn values.
- The converse also holds: if a² + b² = c² for the sides of a triangle, the angle opposite c is a right angle. With floating-point side lengths, test it with a tolerance.

## Applications

- Distances between points in the plane and in space, and the length of a vector.
- The magnitude of a complex number and of a force or velocity from its components.
- Checking that a corner is square in carpentry and surveying with a 3-4-5 triangle.
- Root-mean-square error and standard deviation, which add squared components the same way.

## History

Babylonian tablets such as Plimpton 322 (about 1800 BC) list triples like 3, 4, 5 and much larger ones, and the relation was known in India and China long before a proof was written down. The theorem carries the name of Pythagoras (6th century BC), though no proof by him survives. Euclid's *Elements* (about 300 BC) proves it as Proposition I.47 with a figure of squares on the three sides, and gives the similar-triangles argument in Book VI. The rearrangement proof appears in the Chinese *Zhoubi Suanjing*, and Elisha Loomis's *The Pythagorean Proposition* (1927, revised 1940) collects 367 different proofs.

## References

- Euclid, *Elements*, Book I, Proposition 47, and Book VI, Proposition 31.
- Elisha S. Loomis, *The Pythagorean Proposition*, National Council of Teachers of Mathematics, 1968 (reprint of the 1940 edition).
- Eli Maor, *The Pythagorean Theorem: A 4,000-Year History*, Princeton University Press, 2007.
