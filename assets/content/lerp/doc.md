# Linear interpolation

## Overview

Linear interpolation, usually shortened to **lerp**, blends two values with a single parameter `t`. At `t = 0` you get the start value, at `t = 1` the end value, and every `t` in between lands proportionally far along the straight path between them. It works the same way for plain numbers, points in the plane, colors, or any quantity you can add and scale.

It is the smallest building block in graphics and animation: gradients, tweens, Bézier curves, texture filtering and camera moves are all built from repeated lerps.

## Formula

```math
v(t) = v_{0} + t · (v_{1} − v_{0}),   0 ≤ t ≤ 1
v(t) = (1 − t) · v_{0} + t · v_{1}
```

| Symbol | Meaning |
| --- | --- |
| `v₀` | start value (at t = 0) |
| `v₁` | end value (at t = 1) |
| `t` | blend parameter, usually between 0 and 1 |
| `v(t)` | the blended value |

The two lines are algebraically identical. The first reads as "start, plus a fraction of the gap"; the second as "a weighted average whose weights `1 − t` and `t` always add up to 1".

## Intuition

Picture the segment from `v₀` to `v₁` as a ruler. `t` is the reading on that ruler as a fraction of its length: `t = 0.25` is a quarter of the way along. Because the map from `t` to `v(t)` is linear, **equal steps in t give equal steps in v**, which is why a lerp-driven animation moves at constant speed.

For vectors and colors the formula is applied to each component separately. A point lerp moves along the straight line between the endpoints; a color lerp in RGB walks the straight line through the color cube (which can look muddy in the middle; see the pitfalls below).

## Algorithm

1. Read the two endpoints `v₀`, `v₁` and the parameter `t`.
2. Optionally clamp `t` to `[0, 1]` if extrapolation is not wanted.
3. Return `v₀ + t · (v₁ − v₀)` (or the weighted-average form), component by component.

The inverse operation, **inverse lerp**, recovers `t` from a value: `t = (v − v₀) / (v₁ − v₀)`. Chaining an inverse lerp from one range with a lerp into another range gives a **remap**, the core of every axis scale in a chart.

## Complexity

| Case | Cost |
| --- | --- |
| one value | O(1): one subtraction, one multiplication, one addition |
| a vector with d components | O(d) |

## Parameters in the app

- **t**: the blend parameter; drag it, or press Play to sweep it back and forth.
- **v₀, v₁**: the endpoints of the number line.
- **Speed**: how fast Play sweeps `t`.
- Drag the two handles on the stage to move the endpoints of the point lerp.

## Numerical notes and pitfalls

- **Exact endpoints.** The form `v₀ + t(v₁ − v₀)` may not return exactly `v₁` at `t = 1` in floating point; `(1 − t)v₀ + t·v₁` does, and is also monotonic. Use the weighted form when exact endpoints matter.
- **Extrapolation.** Values of `t` outside `[0, 1]` continue the line past the endpoints. That is useful for prediction, but clamp `t` when you only want blends.
- **Colors.** Lerping gamma-encoded sRGB values darkens the midpoint between saturated colors. For perceptually even gradients, convert to linear RGB or to a perceptual space (Oklab) first, lerp there, then convert back.
- **Angles.** Lerping angles directly takes the long way round when they straddle the wrap-around (350° to 10°). Lerp the shortest signed difference instead, or use quaternion slerp in 3D.
- **Easing.** Feeding `t` through an easing function first, as in `lerp(a, b, ease(t))`, keeps the endpoints but changes the pacing.

## Applications

- Color scales and gradients in charts and maps.
- Tweening positions, sizes and opacity in UI and game animation.
- Estimating a value between two samples (table lookup, sensor readings).
- Texture filtering: bilinear filtering is three lerps.
- De Casteljau's algorithm: a Bézier curve is lerps of lerps.

## History

Interpolation between tabulated values is ancient: astronomers in Babylon and later Ptolemy and Hipparchus interpolated tables of planetary positions. The name *lerp* comes from computer graphics in the 1980s, where it became a standard primitive in shading languages (`mix` in GLSL, `lerp` in HLSL).

## References

- Wikipedia, "Linear interpolation".
- Inigo Quilez, "Remapping functions" (iquilezles.org).
- Björn Ottosson, "A perceptual color space for image processing" (Oklab), 2020.
