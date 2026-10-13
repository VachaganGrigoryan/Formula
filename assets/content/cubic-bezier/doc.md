# Cubic Bézier curve

## Overview

A cubic Bézier curve is a smooth path defined by four points. It starts at P₀, ends at P₃, and leaves and arrives in the directions of the two inner **control points** P₁ and P₂, which pull the curve toward them without (usually) touching it. Moving any of the four points reshapes the whole curve predictably, which is why designers can draw with them by hand.

Cubic Béziers are the standard curve of 2D vector graphics: SVG paths, PostScript and PDF, TrueType-flavored and CFF fonts, CSS easing functions, the pen tool in every illustration program, and curved edges in charts and diagrams. They are low degree (cheap to evaluate), always smooth, and can be chained into longer paths.

## Formula

```math
B(t) = (1 − t)³ P_{0} + 3(1 − t)² t P_{1} + 3(1 − t) t² P_{2} + t³ P_{3},   0 ≤ t ≤ 1
B'(t) = 3(1 − t)² (P_{1} − P_{0}) + 6(1 − t) t (P_{2} − P_{1}) + 3t² (P_{3} − P_{2})
```

| Symbol | Meaning |
| --- | --- |
| P₀, P₃ | endpoints: the curve starts and ends here |
| P₁, P₂ | control points: set the starting and ending directions and how far the curve bulges |
| t | curve parameter, from 0 at P₀ to 1 at P₃ |
| B(t) | the point on the curve at t |
| B'(t) | the velocity (tangent) at t |

The four weights (1 − t)³, 3(1 − t)²t, 3(1 − t)t² and t³ are the cubic **Bernstein polynomials**. They are never negative and always sum to 1, because they are the terms of the binomial expansion of ((1 − t) + t)³ = 1. So B(t) is a weighted average of the control points, and the curve stays inside their convex hull.

## Intuition

### Lerps of lerps

The same curve comes out of repeated linear interpolation, **de Casteljau's algorithm**. For a given t:

```math
A_{i} = lerp(P_{i}, P_{i+1}, t)   for i = 0, 1, 2
C_{i} = lerp(A_{i}, A_{i+1}, t)   for i = 0, 1
B(t) = lerp(C_{0}, C_{1}, t)
```

Four points become three, then two, then one. The app draws this construction: the first colored polyline joins the three A points, the second line joins the two C points, and the last lerp lands on the curve. The final segment C₀C₁ is also the **tangent** to the curve at B(t). Expanding the nested lerps algebraically gives exactly the Bernstein form above.

### What the control points do

At t = 0 only the first weight is nonzero, so the curve starts exactly at P₀; at t = 1 it ends at P₃. The derivative at the ends is B'(0) = 3(P₁ − P₀) and B'(1) = 3(P₃ − P₂): the curve leaves P₀ heading toward P₁ and arrives at P₃ coming from P₂, with a speed proportional to the length of those handles. Long handles make the curve hug the handle direction for longer. The bars under the curve in the app show the four weights at the current t: P₀ dominates near the start, P₃ near the end, and P₁ and P₂ peak at t = 1/3 and t = 2/3 with a weight of 4/9.

## Algorithm

1. Choose t in [0, 1].
2. Compute u = 1 − t and the weights u³, 3u²t, 3ut², t³.
3. Return the weighted sum of P₀ to P₃, component by component.
4. To draw the curve, evaluate it at many t (the app uses 120 steps) and connect the points; or subdivide adaptively until each piece is flat enough.

De Casteljau's version, which also splits the curve in two at t:

```
a0, a1, a2 = lerp(p0, p1, t), lerp(p1, p2, t), lerp(p2, p3, t)
b0, b1     = lerp(a0, a1, t), lerp(a1, a2, t)
c          = lerp(b0, b1, t)
# left half:  p0, a0, b0, c      right half: c, b1, a2, p3
```

## Complexity

| Task | Time | Space |
| --- | --- | --- |
| one point, Bernstein form | O(1): about 10 multiplications per coordinate | O(1) |
| one point, de Casteljau | O(1): 6 lerps per coordinate (O(n²) for degree n) | O(1) |
| polyline with m samples | O(m) | O(m) |
| split at t (de Casteljau) | O(1) | O(1) |
| forward differencing, per sample | O(1): 3 additions per coordinate | O(1) |

## Parameters in the app

- **Drag P₀ to P₃**: move the endpoints (filled handles) and control points (hollow handles). The dashed polyline is the control polygon.
- **t**: the curve parameter; dragging the slider pauses the animation.
- **Speed**: how fast Play sweeps t back and forth between 0 and 1.
- **Play / Pause**: animates t; the bright part of the curve is drawn up to the current t.
- **Show / Hide construction**: toggles de Casteljau's intermediate points and lines.
- **Reset points**: restores the default S-shaped layout.
- The four bars show the Bernstein weights of P₀ to P₃ at the current t.

## Numerical notes and pitfalls

- **t is not arc length.** Equal steps in t do not give equal distances along the curve; points bunch up where the handles are short. For constant-speed motion, build an arc-length table and invert it.
- **Stability.** De Casteljau uses only convex combinations, so it is numerically very stable; the expanded power form a + bt + ct² + dt³ can lose precision. The Bernstein form is a good middle ground.
- **Flattening.** A fixed number of samples wastes points on flat parts and leaves visible corners on tight bends. Adaptive subdivision, which splits until the control polygon is nearly straight, gives an even error.
- **Cusps and loops.** If the control points cross over, the curve can form a loop or a cusp where B'(t) = 0. Offsetting such curves (for stroke outlines) is hard.
- **Joining segments.** Two pieces join smoothly (G1) only if the shared endpoint and the handles on either side are collinear; equal handle lengths give C1.
- **Circles cannot be exact.** A Bézier is a polynomial and cannot trace a circle exactly. A quarter circle of radius r is approximated with handles of length about 0.5523 r, with a radius error under 0.03%.

## Applications

- Vector graphics and fonts: SVG and PDF paths, PostScript, CFF/OpenType outlines (TrueType uses quadratics).
- Animation timing: CSS `cubic-bezier()` easing curves and keyframe interpolation in animation tools.
- Charts: smooth line charts and curved links in Sankey, chord and node-link diagrams.
- Industrial design and CAD: car body and product surfaces, as Bézier patches and their generalization, NURBS.
- Robotics and games: smooth paths for vehicles, cameras and projectiles.

## History

The curves are named after Pierre Bézier, an engineer at Renault, who published them in the 1960s for designing car bodies in the UNISURF system. Paul de Casteljau had developed the same curves earlier, around 1959, at Citroën, together with the recursive algorithm now named after him, but Citroën kept his work confidential, so Bézier's name stuck. The weights date back further: Sergei Bernstein introduced Bernstein polynomials in 1912 for a constructive proof of the Weierstrass approximation theorem. Cubic Béziers reached every desktop through PostScript (1984) and later SVG and OpenType.

## References

- Gerald Farin, "Curves and Surfaces for CAGD: A Practical Guide", 5th edition, Morgan Kaufmann, 2002.
- Pierre Bézier, "Numerical Control: Mathematics and Applications", Wiley, 1972.
- Wikipedia, "Bézier curve" and "De Casteljau's algorithm".
- Mike "Pomax" Kamermans, "A Primer on Bézier Curves" (online book).
