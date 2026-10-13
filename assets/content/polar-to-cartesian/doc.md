# Polar to Cartesian

## Overview

Polar coordinates name a point by **how far** it is from a fixed origin (the pole) and **in which direction** it lies, as an angle θ measured counterclockwise from a fixed ray (the polar axis, usually the positive x axis). Cartesian coordinates name the same point by its horizontal and vertical offsets x and y. Converting from one to the other takes two lines of trigonometry.

The conversion matters because many things are naturally described by distance and direction (a pie slice, a radar sweep, a lidar return, a joystick, a planet's orbit) while screens, plotting libraries and most geometry code work in x and y. Every radial chart ends with exactly this step: compute (r, θ) from the data, then turn it into pixels.

## Formula

```math
x = r · cos(θ)
y = r · sin(θ)
r = √(x² + y²)
θ = atan2(y, x)
```

| Symbol | Meaning |
| --- | --- |
| r | radius: distance from the origin |
| θ | angle in radians, counterclockwise from the positive x axis |
| x, y | Cartesian coordinates of the same point |
| `atan2(y, x)` | two-argument arctangent: the angle of the vector (x, y), in (−π, π] |

The first two lines are the forward conversion; the last two are its inverse. The app keeps θ in [0, 2π) by adding 2π to negative results of `atan2`.

## Intuition

Draw the ray from the origin at angle θ and walk a distance r along it. Drop a perpendicular to the x axis: you get a right triangle with hypotenuse r and angle θ at the origin. By the definitions of sine and cosine, the side along the x axis (adjacent) is r cos θ and the vertical side (opposite) is r sin θ. That is the whole formula. In the app these two sides are drawn as the thick colored bars on the axes, and the dashed lines are the perpendiculars.

Another way to see it: (cos θ, sin θ) is the point on the **unit circle** at angle θ, a direction of length 1. Multiplying by r stretches it to the right length. The inverse follows from the same triangle: Pythagoras gives r² = x² + y², and the slope y / x is tan θ.

### Negative radius and rose curves

The formula does not care about the sign of r. A negative r walks backwards along the ray and lands on the opposite side, at angle θ + π. The rose curve r = cos(kθ) uses this: as θ sweeps once around, the radius swings between +1 and −1, drawing petals. For odd k the curve has k petals and is traced twice per turn (the second pass lands on the first because of the negative radius); for even k it has 2k petals.

## Algorithm

1. Read r and θ. If θ is in degrees, convert: θ = degrees · π / 180.
2. Compute c = cos θ and s = sin θ (many libraries offer a combined `sincos`).
3. Return x = r · c and y = r · s.
4. To draw on a screen whose y axis points down, map `px = cx + scale * x` and `py = cy - scale * y`.

The inverse, used when the pointer is dragged on the stage:

```
r     = hypot(x, y)
theta = atan2(y, x)
if theta < 0: theta = theta + 2 pi
```

A polar curve r = f(θ) is drawn by sampling θ densely and converting each sample:

```
for j in 0..=steps:
    theta = 2 pi * j / steps
    r     = f(theta)            # e.g. cos(k * theta)
    vertex(r * cos(theta), r * sin(theta))
```

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| polar to Cartesian, one point | O(1): one sin, one cos, two multiplications | O(1) |
| Cartesian to polar, one point | O(1): one hypot, one atan2 | O(1) |
| sampling a polar curve with n points | O(n) | O(n) for the vertex list |

## Parameters in the app

- **Point / Rose curve r = cos kθ**: switches between a free point and the rose curve, whose radius is computed from the angle.
- **r** (point mode): the radius, from 0 at the center to 1 at the outer ring.
- **k** (rose mode): the rose parameter, 1 to 9; odd k gives k petals, even k gives 2k.
- **θ**: the angle in degrees; moving the slider pauses the animation.
- **Speed**: how fast Play sweeps θ around the circle.
- **Play / Pause**: starts or stops the sweep.
- **Click or drag on the stage**: converts the pointer position back to polar form with `atan2` and `hypot`, setting θ (and r in point mode).
- The grid shows rings of equal r at 0.25 steps and spokes of equal θ every 30°. The readout prints r, θ, x and y.

## Numerical notes and pitfalls

- **Degrees versus radians.** Trigonometric functions in almost every language take radians. Feeding degrees is the most common bug; convert once, at the boundary.
- **Screen y points down.** In pixel coordinates y grows downward, so a counterclockwise angle needs `py = cy - r * sin(theta)`. Forgetting the minus sign mirrors everything and turns counterclockwise into clockwise.
- **Use atan2, not atan.** `atan(y / x)` divides by zero on the y axis and cannot tell opposite quadrants apart, since (1, 1) and (−1, −1) have the same ratio. `atan2(y, x)` uses both signs. Mind the argument order: y first.
- **Angle range.** `atan2` returns (−π, π]. If you need [0, 2π), add 2π to negative values, or use a Euclidean remainder. In JavaScript `%` keeps the sign of its left operand, so write `((a % TAU) + TAU) % TAU`.
- **The origin.** At r = 0 the angle is undefined; `atan2(0, 0)` returns 0 by convention, which is harmless but arbitrary.
- **Exact zeros.** cos(π / 2) is about 6e-17 in double precision and about −4e-8 in single precision, not 0. Do not test computed coordinates for exact equality.
- **Wrap-around.** Interpolating or averaging angles near 0 and 2π directly gives nonsense (the mean of 350° and 10° is not 180°). Work with the shortest signed difference, or average the unit vectors instead.
- **Uniform sampling.** Picking r uniformly in [0, 1] crowds random points near the center of a disk; use r = √u with u uniform.

## Applications

- Pie and donut charts, radar (spider) charts, Nightingale rose diagrams, circular heatmaps and radial tree layouts.
- Robotics and sensing: lidar, radar and sonar report range and bearing, which are converted to x, y before mapping.
- Navigation and surveying: a distance and a compass bearing give a position offset.
- Games and UI: analog stick input, orbiting cameras, radial menus, placing objects on a circle.
- Complex numbers: z = r(cos θ + i sin θ), the polar form used for multiplying, rotating and taking roots.
- Physics and engineering: central-force orbits, antenna radiation patterns, phasors in AC circuits.

## History

Measuring position by angle and distance is ancient. Hipparchus built chord tables for astronomy in the second century BC, and Archimedes described his spiral (r = aθ in modern terms) in *On Spirals* around 225 BC. Polar coordinates as a coordinate system for curves appear in the 1630s and 1640s with Grégoire de Saint-Vincent and Bonaventura Cavalieri. Isaac Newton treated them as one of several systems in his *Method of Fluxions* (written around 1671), and Jacob Bernoulli used them in *Acta Eruditorum* in 1691. The name *polar coordinates* is credited to Gregorio Fontana in the 18th century and reached English through George Peacock's 1816 translation of Lacroix. Rose curves were studied by Guido Grandi in the 1720s, who called them *rhodonea*. The two-argument `atan2` dates back at least to FORTRAN.

## References

- Wikipedia, "Polar coordinate system".
- Wikipedia, "Atan2".
- Wikipedia, "Rose (mathematics)".
- Julian Lowell Coolidge, "The Origin of Polar Coordinates", American Mathematical Monthly 59 (1952).
- James Stewart, "Calculus", chapter on parametric equations and polar coordinates.
