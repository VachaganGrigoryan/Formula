# Perlin noise flow field

## Overview

A flow field assigns a direction to every point of the plane, and particles dropped into it simply follow the arrow under them. Here the direction comes from **Perlin noise**, a smooth pseudo-random function: sample the noise at a point, turn the value into an angle, and step the particle a little way along that angle. Thousands of particles leave thin trails that fade slowly, so the hidden field draws itself as streaks, swirls and eddies, like wind over grass or iron filings around a magnet.

Perlin noise matters far beyond this picture. It is the standard building block of procedural graphics: terrain heights, clouds, fire, water, marble and wood grain in games and film are all made by sampling and layering it.

## Formula

```math
θ(x, y) = N(x · s, y · s, t) · 2π · k
p_{n+1} = p_{n} + v Δt · (cos θ, sin θ)
fade(u) = 6u^{5} − 15u^{4} + 10u^{3}
```

| Symbol | Meaning |
| --- | --- |
| `N(x, y, t)` | improved Perlin noise, rescaled from about -1..1 to 0..1 |
| `s` | zoom: noise features per pixel; larger is busier |
| `k` | turns: how many full rotations the angle covers as N goes from 0 to 1 |
| `t` | the third noise coordinate, used as time so the field slowly changes |
| `p`, `v`, Δt | particle position, speed in pixels per second, frame time |
| `fade` | the quintic easing curve that blends the corners of a noise cell |

The noise inside a lattice cell is a trilinear blend of eight corner contributions. Each corner contributes the dot product of a pseudo-random gradient vector with the offset from that corner to the sample point, and the blend weights come from `fade` applied to the fractional coordinates.

## Intuition

Plain random numbers at every pixel would point particles every which way and give static, not flow. Perlin noise is *coherent*: nearby points get nearby values, so nearby particles get nearly the same heading and move together in streams. The zoom `s` sets how far apart the features are: small `s` gives broad, lazy currents, large `s` tight curls.

Gradient noise is zero at every lattice point and rises or falls away from it according to that point's random gradient, so it has no blocky look. The quintic `fade` curve has zero first and second derivatives at 0 and 1, which is what removes the visible grid seams of Perlin's original cubic version.

Multiplying by 2π · k with `k` above 1 means the noise sweeps the angle through more than a full turn, so the same small change in N swings the direction further, producing loops and whirlpools. Moving the third coordinate `t` forward slowly morphs the whole field without breaking its smoothness.

Trails come from not clearing the screen: each frame the old image is blended a few percent toward the background, so recent motion is bright and older motion fades out.

## Algorithm

1. Build a permutation table: shuffle 0..255 with a seed and repeat it to 512 entries.
2. Scatter particles at random positions, each with a random lifetime of a few seconds.
3. Each frame, fade the trail image a little toward the background.
4. For each particle, compute the angle from the noise at its position, step it forward, and draw a line from the old position to the new one, colored by angle.
5. When a particle leaves the canvas or its lifetime runs out, respawn it at a random position.
6. Advance the noise time coordinate by a small amount.

```
noise(x, y, z):
    X, Y, Z = floor(x) & 255, floor(y) & 255, floor(z) & 255
    x, y, z = x - floor(x), y - floor(y), z - floor(z)
    u, v, w = fade(x), fade(y), fade(z)
    hash the 8 cell corners with perm[perm[perm[X] + Y] + Z] ...
    blend grad(corner hash, offset) with lerps by u, then v, then w

grad(h, x, y, z):            # one of 12 edge directions
    h = h & 15
    a = x if h < 8 else y
    b = y if h < 4 else (x if h in (12, 14) else z)
    return (a if h & 1 == 0 else -a) + (b if h & 2 == 0 else -b)
```

## Complexity

| Quantity | Cost |
| --- | --- |
| one noise sample | O(1): 8 gradients, 7 lerps |
| one frame, P particles | O(P) noise samples and short lines |
| trail fade, W x H canvas | O(W · H) per frame |
| memory | O(P + W · H), plus the 512-entry table |

## Parameters in the app

- **Particles**: how many particles move at once (100 to 5000).
- **Zoom s**: the scale of the noise. Small values give wide, smooth currents; large values give tight turbulence.
- **Turns k**: how many full rotations the angle spans; higher values give more loops and vortices.
- **Speed v**: particle speed in pixels per second.
- **Trail fade**: how much of the trail image fades toward the background each frame; small values leave long trails.
- **Field change**: how fast the noise's time coordinate advances, so the field itself drifts.
- **Color**: *By angle* maps the heading to a hue; *Ink* draws every particle in one color.
- **Play / Pause**, **Clear** (wipe trails), **New noise** (reseed the permutation table), **Show vectors** (draw the field as a grid of arrows).
- Hover anywhere to read N and θ at that point.

## Numerical notes and pitfalls

- **Integer lattice values.** Noise is exactly 0 at integer coordinates. Sampling with `s = 1` on integer pixels returns zeros everywhere; always scale coordinates by a small, non-integer factor.
- **Range.** Improved Perlin noise in 3D stays within about -1..1 but rarely reaches the ends, and its values cluster around 0. Mapped to an angle with `k = 1`, some directions are much more common than others; that is part of the look, not a bug.
- **Negative coordinates.** Use `floor`, not truncation, so cells are consistent on both sides of 0, and mask with `& 255` after flooring.
- **Fading by translucent overlay.** Drawing a translucent background rectangle on an 8-bit canvas never quite reaches the background color: rounding leaves faint permanent ghost trails. Use a stronger fade, a floating-point buffer, or an occasional full clear.
- **Gaps in trails.** At high speed a particle jumps several pixels per frame; draw line segments, not dots.
- **Clumping.** Particles accumulate in sinks of the field. Giving each one a finite lifetime and respawning it at random keeps the coverage even.
- **Frame-rate independence.** Scale the step by the frame time `dt` and the fade by `1 - (1 - f)^(60 dt)` so the picture looks the same at any frame rate.

## Applications

- Procedural textures and terrain: clouds, marble, wood, fire, smoke and height maps.
- Generative art and plotter drawings built from flow-field streamlines.
- Particle effects in games: dust, embers, snow and magic drifting on noise-driven wind.
- Visualizing vector data such as wind maps, ocean currents and magnetic fields.
- Animation: natural-looking camera shake and idle motion.

## History

Ken Perlin developed the noise function in 1983, after working on the computer graphics for the film *Tron* (1982) left him frustrated with their machine-like look, and published it in "An Image Synthesizer" at SIGGRAPH 1985. He received an Academy Award for Technical Achievement for it in 1997. In 2002 he published "Improving Noise", which replaced the cubic blend with the quintic `fade` and the random gradients with twelve fixed edge directions; that version is the one used here. Flow fields drawn with particle trails became a staple of generative art through creative coding tools such as Processing, popularized by writers like Daniel Shiffman and Tyler Hobbs.

## References

- Ken Perlin, "An Image Synthesizer", SIGGRAPH 1985.
- Ken Perlin, "Improving Noise", SIGGRAPH 2002.
- David S. Ebert, F. Kenton Musgrave, Darwyn Peachey, Ken Perlin and Steven Worley, *Texturing and Modeling: A Procedural Approach*.
- Daniel Shiffman, *The Nature of Code*, chapter on autonomous agents and flow fields.
- Tyler Hobbs, "Flow Fields" (essay on generative art).
- Wikipedia, "Perlin noise".
