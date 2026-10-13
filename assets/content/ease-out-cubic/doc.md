# Cubic ease-out

## Overview

An **easing function** reshapes the progress of an animation. Linear progress moves at constant speed and starts and stops abruptly, which looks mechanical. **Cubic ease-out**, `f(t) = 1 − (1 − t)³`, starts fast and glides to a gentle stop, the way a sliding object comes to rest under friction. It is one of the most common curves in user interfaces because motion that decelerates into place reads as "arriving".

The function maps animation progress `t` (elapsed time divided by duration, from 0 to 1) to eased progress `f(t)`, also from 0 to 1. The eased value then drives an ordinary interpolation, `lerp(start, end, f(t))`, so the endpoints are unchanged and only the pacing differs. The app shows the curve with its speed, a race between a linear and an eased dot, and a bar chart growing into place.

## Formula

```math
f(t) = 1 − (1 − t)^{3},   0 ≤ t ≤ 1
f'(t) = 3 (1 − t)^{2}
v(τ) = v_{0} + (v_{1} − v_{0}) · f(τ / T)
```

| Symbol | Meaning |
| --- | --- |
| `t` | linear progress, elapsed time over duration, clamped to [0, 1] |
| `f(t)` | eased progress |
| `f'(t)` | speed of the eased motion relative to linear (the slope of the curve) |
| `τ` | time since the animation started |
| `T` | duration of the animation |
| `v₀`, `v₁` | start and end values of the animated property |

Expanded, `f(t) = 3t − 3t² + t³`. Useful values: `f(0) = 0`, `f(0.5) = 0.875`, `f(1) = 1`; the speed is 3 at the start and 0 at the end.

## Intuition

The speed `f'(t) = 3(1 − t)²` tells the whole story. At the start the eased motion runs three times faster than linear; the speed then falls off quadratically and reaches exactly zero at `t = 1`, so the motion stops without a jolt. Half-way through the time, the object is already 87.5% of the way there.

The dots in the app's race make this visible. Each dot marks where the object is after another tenth of the duration. On the linear lane the dots are evenly spaced; on the ease-out lane they are spread wide at the start and bunched up at the end, where the object covers less and less ground per time slice.

### Relation to ease-in

Cubic **ease-in** is `g(t) = t³`: slow start, fast finish. Ease-out is its mirror image through the center of the unit square: `f(t) = 1 − g(1 − t)`. Playing an ease-in backwards in time gives an ease-out. Ease-in reads as "leaving", ease-out as "arriving"; ease-in-out joins the two halves.

### A Bézier curve in disguise

Written in the Bernstein basis, `f(t) = 0·(1 − t)³ + 1·3t(1 − t)² + 1·3t²(1 − t) + 1·t³`: a one-dimensional cubic Bézier curve with control values 0, 1, 1, 1. In CSS terms it is exactly `cubic-bezier(1/3, 1, 2/3, 1)`, usually rounded to `cubic-bezier(0.33, 1, 0.68, 1)`. Changing the power changes the strength: quadratic ease-out `1 − (1 − t)²` starts at speed 2, quartic at speed 4.

## Algorithm

1. When the animation starts, record the start value `v₀`, the target `v₁` and the start time.
2. Each frame, compute `t = clamp((now − start) / T, 0, 1)`.
3. Compute `u = 1 − t` and `e = 1 − u·u·u`.
4. Set the property to `v₀ + (v₁ − v₀)·e`.
5. When `t` reaches 1, stop, or (when looping) hold and start the next animation.

To retarget mid-flight, as the app's bars do, take the current value as the new `v₀` and restart the clock.

```
t = clamp(elapsed / duration, 0, 1)
u = 1 - t
e = 1 - u * u * u
value = lerp(from, to, e)
```

## Complexity

| Operation | Time | Space |
| --- | --- | --- |
| one evaluation of f(t) | O(1): 2 subtractions, 2 multiplications | O(1) |
| animating n values per frame | O(n) | O(n) for start and end values |

## Parameters in the app

- **Duration** (0.3 to 4 s): how long one animation takes; `t` is elapsed time divided by it.
- **Replay**: starts a new animation of the bars toward fresh random heights, from wherever they are now.
- **Loop**: after each animation, hold the result for 0.8 s and start another.
- **Ease the bars**: switches the bar chart between eased and linear motion, for comparison.
- The left graph shows `f(t)` against the dashed linear diagonal, plus the speed `f'(t) / 3`; the moving dot marks the current `t` and `f(t)`.

## Numerical notes and pitfalls

- **Clamp t.** Outside [0, 1] the polynomial keeps going: `f(t) > 1` for `t > 1` and `f(t) < 0` for `t < 0`, so a late frame overshoots the target. Clamp `t` first.
- **Use elapsed time, not frame counts.** Advancing `t` by a fixed step per frame makes the duration depend on frame rate. Compute `t` from a clock, and cap very large frame gaps after the tab was hidden.
- **Multiply, do not pow.** `u*u*u` is exact enough and cheaper than `pow(u, 3)`; at `t = 1` it gives exactly `f = 1`.
- **Retargeting.** Restarting from the old start value makes the object jump. Start from the current value; note that the speed then resets to its fast start, which can look like a kick if retargeting happens every frame.
- **Not the same as exponential smoothing.** `x += (target − x)·k` per frame also decelerates, but never finishes and depends on frame rate unless `k = 1 − exp(−rate·dt)`.
- **Accessibility.** Honor reduced-motion settings by shortening or skipping animations.

## Applications

- UI transitions: menus, panels and dialogs sliding or fading into place.
- Data visualization: bars growing, points moving to new positions after a filter change, counters ticking up.
- Scrolling to an anchor, zooming maps and moving cameras in games.
- CSS transitions and animation libraries: D3's `d3.easeCubicOut`, GSAP's `power2.out`, and Android's DecelerateInterpolator with factor 1.5.

## History

Hand-drawn animation has long used "slow in and slow out", one of the twelve principles that Disney animators Frank Thomas and Ollie Johnston set out in *The Illusion of Life* (1981): drawings are spaced more closely near key poses so motion eases into and out of them. In computer animation, Robert Penner's easing equations, published around 2001 for Macromedia Flash and in his 2002 book, gave the family of quadratic, cubic, quartic and other curves the names still used today, including *easeOutCubic*. CSS Transitions later standardized easing through `cubic-bezier()` timing functions, the Bézier curves introduced by Pierre Bézier and Paul de Casteljau for car design.

## References

- R. Penner, *Robert Penner's Programming Macromedia Flash MX*, chapter "Motion, Tweening, and Easing", 2002.
- F. Thomas and O. Johnston, *The Illusion of Life: Disney Animation*, 1981.
- W3C, "CSS Easing Functions Level 1".
- Wikipedia, "Twelve basic principles of animation".
- D3 documentation, "d3-ease".
