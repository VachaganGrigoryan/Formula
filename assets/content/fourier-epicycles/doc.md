# Drawn by circles

## Overview

Any closed outline can be traced by a chain of spinning circles. Walk once around the outline at a steady pace and record the pen position as two periodic functions, x(t) and y(t). Each can be written as a **Fourier series**: a constant plus cosines and sines whose frequencies are whole multiples of one turn. Each term is a point going round a circle k times per turn; adding the terms nose to tail gives a chain of circles riding on circles, the **epicycles** of ancient astronomy, whose tip draws the shape.

Keeping only the first N terms gives the best smooth approximation with that many circles: the slow, large circles carry the overall shape, the fast, small ones the detail. With a handful of circles only a rough blob survives; as N grows, ears, fins and tails appear.

## Formula

```math
x_{N}(t) = a_{0} + Σ_{k=1…N} (a_{k} cos kt + b_{k} sin kt)
y_{N}(t) = c_{0} + Σ_{k=1…N} (c_{k} cos kt + d_{k} sin kt)
a_{k} = (2/M) Σ_{j=0…M−1} x_{j} cos(2πkj/M),   b_{k} = (2/M) Σ_{j=0…M−1} x_{j} sin(2πkj/M)
a_{k} cos kt + b_{k} sin kt = r_{k} cos(kt − φ_{k}),   r_{k} = √(a_{k}² + b_{k}²),   φ_{k} = atan2(b_{k}, a_{k})
```

| Symbol | Meaning |
| --- | --- |
| t | position along the outline, as an angle from 0 to 2π |
| M | number of samples taken evenly along the outline (512 in the app) |
| `x[j]`, `y[j]` | the samples, j = 0 … M − 1 |
| a₀, c₀ | the means of x and y: the center the chains start from |
| `a[k]`, `b[k]` | cosine and sine coefficients of harmonic k for x (`c[k]`, `d[k]` for y) |
| `r[k]`, `phi[k]` | radius and phase φ of the circle for harmonic k |
| N | number of terms kept |

The coefficients for y are computed the same way from the y samples. The last line rewrites each cosine-sine pair as one circle of radius `r[k]` turning at k turns per period, which is what the app draws.

## Intuition

### One term is one circle

The point (r cos(kt − φ), r sin(kt − φ)) moves around a circle of radius r, k times per turn, starting at angle −φ. Its horizontal shadow is exactly the term r cos(kt − φ). Chain several such arms, each starting where the previous one ends, and the horizontal position of the last tip is the sum of the shadows, the partial sum for x at time t. The app draws this **x chain** along the top. The **y chain** on the left is the same idea turned 90°, so that the vertical position of its tip is the partial sum for y. Dashed lines from the two tips meet at the pen.

Addition does not care about order, so the circles can be stacked in any order. The app sorts each chain **largest first**, so the big, slow circles sit at the root and the small, fast ones cluster at the tip.

### Where the coefficients come from

Over M evenly spaced samples, cosines and sines of different whole frequencies are orthogonal: the sum of cos(2πkj/M) · cos(2πlj/M) over j is zero unless k = l. Multiplying the data by cos(2πkj/M) and summing therefore isolates harmonic k: this is the discrete Fourier transform (DFT). Truncating after N terms gives the N-term curve with the least squared error to the samples, and Parseval's theorem says that this error is proportional to the sum of the squared radii of the dropped circles.

### Why detail melts away

For a smooth outline the radii shrink quickly with k. Corners make them shrink only like 1/k², so sharp features need many circles. Removing the high terms is a low-pass filter: corners round off, thin parts thicken, and at N = 1 every shape becomes an ellipse.

## Algorithm

1. Take the outline as a closed polygon.
2. Resample it to M points evenly spaced by arc length, so that t moves at constant speed along the outline.
3. Normalize: center the bounding box and scale the longer side to 1.
4. For k = 1 … N, compute the DFT sums for x and y and convert each pair to a radius and a phase.
5. Each frame, advance t. For each chain, start at the mean, sort the first N terms by radius, and add the arms one by one, drawing each circle and arm.
6. The pen is (tip of the x chain, tip of the y chain). Append it to the trail; clear the trail after a full turn.

```
for k in 1..=N:
    a = b = 0
    for j in 0..M:
        a += x[j] * cos(2 pi k j / M);  b += x[j] * sin(2 pi k j / M)
    r[k], phi[k] = hypot(a, b) * 2/M, atan2(b, a)
x(t) = mean + sum over k of r[k] * cos(k t - phi[k])
```

## Complexity

| Task | Time | Space |
| --- | --- | --- |
| resample P vertices to M points | O(P + M) | O(M) |
| N harmonics by the direct DFT | O(N · M) | O(N) |
| all harmonics by the FFT | O(M log M) | O(M) |
| pen position for one t | O(N) | O(1) |
| sort the circles by radius | O(N log N) | O(N) |
| reconstructed curve at S points | O(N · S) | O(S) |

In the app, M = 512 and up to 200 harmonics: about 400 000 multiply-adds per animal, computed once and cached.

## Parameters in the app

- **Terms N**: how many harmonics (circles per chain) are used, 1 to 200. Fewer terms give a smoother, rounder outline.
- **Speed**: how fast t advances; at speed 1 one trip around the outline takes about 48 seconds.
- **Play / Pause**: stops or resumes the pen.
- **Random animal**: picks another outline and grows N from 1 to the slider value over about 2 seconds, so you can watch the shape emerge.
- **Show / Hide outline**: toggles the dashed original outline.
- **Choose an animal** (details panel): search by name or filter by group.
- On the stage, the filled shape is the full N-term curve, the bright trail is what the pen has drawn this turn, and the large number is N.

## Numerical notes and pitfalls

- **Sample evenly.** With unevenly spaced vertices, t speeds up and slows down along the outline and the spectrum is distorted. Resample by arc length first.
- **Close the loop.** If the last point is far from the first, the coordinates jump at the seam and the truncated series rings there (the Gibbs phenomenon, an overshoot of about 9% of the jump that never goes away).
- **Nyquist limit.** With M samples, only harmonics up to M / 2 are meaningful; higher k just alias lower ones. The app stops at 200 of 256.
- **Cost.** The direct DFT is O(N · M); for large M precompute the sines and cosines or use an FFT, and accumulate in double precision.
- **Conventions.** Mixing up atan2(b, a) with atan2(a, b), or cos(kt − φ) with cos(kt + φ), mirrors or scrambles the drawing. Pick one convention and test it on a circle, which must come out as one single harmonic.
- **Tiny circles.** Sub-pixel circles cost time and add nothing; the app skips drawing circles under 0.6 pixels.

## Applications

- Elliptic Fourier descriptors: comparing and classifying outlines of leaves, fossils, fish otoliths and cells, independent of size and rotation.
- Contour smoothing: dropping high harmonics removes digitizing noise from traced outlines.
- Compression: JPEG (cosine transform) and MP3 or AAC (modified cosine transform) keep the strong low-frequency coefficients and drop the rest.
- Shape morphing: blending the coefficients of two outlines gives smooth in-between shapes.

## History

Circles on circles first modeled the sky: Hipparchus used epicycles in the 2nd century BC, and Ptolemy's *Almagest* (around 150 AD) predicted planetary positions with them, in effect a short Fourier series. The name comes from the Greek *epi* (upon) and *kyklos* (circle). Joseph Fourier claimed in 1807, while studying heat conduction, that arbitrary functions can be expanded in sines and cosines, and published it in *Théorie analytique de la chaleur* (1822). James Cooley and John Tukey published the fast Fourier transform in 1965 (Gauss had a version around 1805). Frank Kuhl and Charles Giardina introduced elliptic Fourier descriptors of closed contours in 1982. Drawing with epicycles was popularized by online animations such as 3Blue1Brown's 2019 video.

## References

- Wikipedia, "Fourier series", "Discrete Fourier transform" and "Deferent and epicycle".
- Joseph Fourier, "Théorie analytique de la chaleur", 1822.
- Frank P. Kuhl and Charles R. Giardina, "Elliptic Fourier features of a closed contour", Computer Graphics and Image Processing 18 (1982).
- James W. Cooley and John W. Tukey, "An algorithm for the machine calculation of complex Fourier series", Mathematics of Computation 19 (1965).
- Grant Sanderson (3Blue1Brown), "But what is a Fourier series? From heat flow to drawing with circles", video, 2019.
