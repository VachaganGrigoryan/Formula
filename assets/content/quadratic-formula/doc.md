# Quadratic formula

## Overview

A quadratic equation a x² + b x + c = 0 asks where the parabola y = a x² + b x + c meets the x axis. The quadratic formula answers it for any coefficients, and the discriminant b² − 4ac says in advance how many real answers there are: two, one (a double root where the parabola just touches the axis) or none.

The tool draws the parabola with its roots, its vertex and its axis of symmetry. The Derive tab derives the formula by completing the square, and plugs the current a, b and c into it step by step.

## Formula

```math
a x² + b x + c = 0,   a ≠ 0
x = (−b ± √(b² − 4ac)) / 2a
Δ = b² − 4ac
```

| Symbol | Meaning |
| --- | --- |
| a | the x² coefficient: how wide the parabola is, and whether it opens up (a > 0) or down |
| b | the x coefficient: with a it sets the axis of symmetry x = −b / 2a |
| c | the constant: where the parabola crosses the y axis |
| Δ | the discriminant: positive gives two roots, zero one double root, negative none |
| x | a root: a value of x where the expression is zero |

## Intuition

Every parabola is symmetric about the vertical line through its vertex, x = −b / 2a. The two roots, when they exist, sit at the same distance on either side of that line, so the formula is "the axis of symmetry, plus or minus a half-width". The half-width is √Δ / 2a: when Δ shrinks to zero the two roots slide together into the vertex, and when Δ is negative the vertex is on the wrong side of the axis and the parabola never reaches it.

Completing the square makes this exact. Adding (b / 2a)² to both sides turns x² + (b/a) x into the perfect square (x + b/2a)², and an equation of the form (something)² = number is solved by a square root.

## Algorithm

1. If a = 0 the equation is linear, b x + c = 0, with the single root −c / b (or none when b = 0 too).
2. Compute Δ = b² − 4ac.
3. If Δ < 0 there are no real roots; if Δ = 0 there is one, −b / 2a.
4. Otherwise compute q = −(b + sign(b) √Δ) / 2 and take the roots q / a and c / q.

Step 4 is the textbook formula rearranged so that b and √Δ are never subtracted when they are nearly equal (see the pitfalls below).

## Complexity

A constant number of operations: one square root, a few multiplications and two divisions. Drawing the parabola samples it once per screen point, with extra samples where it bends.

## Parameters in the app

- **a, b, c**: sliders from −5 to 5. Presets set two roots, a double root, no real roots, or a parabola that opens down.
- **Stage**: the parabola, the root markers with their values, the vertex and the dashed axis of symmetry. A caption states the discriminant case. Drag to pan, scroll to zoom, *Reset view* returns.
- **Derive tab**: "Plug in current values" works out Δ, x₁ and x₂ for the current coefficients one operation at a time; "Completing the square" derives the formula in seven steps.

## Numerical notes and pitfalls

- When b² is much larger than 4ac, −b + √(b² − 4ac) subtracts two nearly equal numbers and loses most of its digits. The form q = −(b + sign(b)√Δ)/2, x = q/a and x = c/q avoids that subtraction; the app and the exported code use it.
- Testing Δ = 0 exactly is fragile in floating point. The app treats |Δ| below 1e-12 as zero.
- With a very small but nonzero a one root goes off towards infinity: the equation is close to linear.
- Plugging values into the textbook formula with Δ < 0 takes the square root of a negative number, which the Derive tab shows as "undefined" for real numbers. Over the complex numbers the two roots are −b/2a ± i√(−Δ)/2a.

## Applications

- Physics: when a thrown object lands, h₀ + v t − g t²/2 = 0.
- Graphics: intersecting a ray with a sphere or a circle reduces to a quadratic in the ray parameter.
- Economics: break-even points where a quadratic profit curve crosses zero.
- Collision detection: the time at which two objects moving at constant velocity come within a given distance.

## History

Babylonian clay tablets from about 1800 BC solve problems equivalent to quadratic equations with a recipe that amounts to completing the square. Euclid gave geometric constructions, and Brahmagupta stated a rule close to the modern formula in 628, including negative quantities. Al-Khwarizmi's book on al-jabr (about 820) classified the cases and justified them by completing squares geometrically, which is where the method gets its name. The symbolic formula in its modern form comes from the notation of Viète and Descartes in the 16th and 17th centuries.

## References

- Victor J. Katz, *A History of Mathematics*, 3rd ed., Pearson, 2009, chapters 1 and 9.
- William H. Press et al., *Numerical Recipes*, 3rd ed., Cambridge University Press, 2007, section 5.6 (quadratic and cubic equations).
- David Goldberg, "What Every Computer Scientist Should Know About Floating-Point Arithmetic", ACM Computing Surveys 23(1), 1991.
