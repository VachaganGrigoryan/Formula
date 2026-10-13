# Lissajous figures

## Overview

A Lissajous figure is the path traced by a point that oscillates sinusoidally in two perpendicular directions at once: one sine wave drives the horizontal position, another drives the vertical one. When the two frequencies are in a whole-number ratio such as 3:2, the path closes into a knot-like loop; the ratio sets the number of lobes and the phase difference between the waves twists the knot.

These curves are the classic picture on an oscilloscope in X-Y mode, where they let an engineer read off the frequency ratio and phase difference of two signals at a glance. They also turn up in audio phase meters, vibration analysis, laser shows and logo design.

## Formula

```math
x(t) = A sin(a t + δπ)
y(t) = B sin(b t)
```

| Symbol | Meaning |
| --- | --- |
| A, B | amplitudes: half the width and half the height of the figure |
| a, b | angular frequencies of the horizontal and vertical waves |
| δ | phase offset of x, in units of π (δ = 0.5 means a quarter turn) |
| t | time, or simply the curve parameter |
| x(t), y(t) | the position of the point |

For whole numbers a and b the curve is closed with period 2π / gcd(a, b). The app takes a and b from 1 to 9; reducing the ratio (6:4 is the same figure as 3:2, traced twice) does not change the picture.

## Intuition

Picture two independent pendulums or two speakers: one pushes the pen left and right, the other up and down. The figure fits in the rectangle [−A, A] × [−B, B] and touches its vertical sides a times and its horizontal sides b times per period (for coprime a, b and a generic phase). So you can **count touches to read the ratio**: the number of times the curve touches a side edge is a, and the number of times it touches the top edge is b.

The phase decides how the two waves line up.

- With a = b, the figure is an ellipse. At δ = 0 it collapses to a diagonal line (the waves are in step), at δ = 0.5 it is an axis-aligned ellipse, or a circle when A = B, and at δ = 1 it is the other diagonal.
- With other ratios, certain phases make the curve fold back on itself into an open arc that is traced back and forth: with a 1:2 ratio, A = B = 1 and δ = 0.25, it is the parabola y = 2x² − 1, the Chebyshev polynomial T₂. Other phases open it into the full knot.

### Why it seems to turn in 3D

When δ increases steadily, every point of the figure slides along the knot, and the result looks like a wire curve rotating about a vertical axis. That is no accident: the figure is the side view of a curve wound on a cylinder, x = A sin(at + δπ) being the projection of a point going round the cylinder. Shifting δ is the same as rotating the cylinder. On an oscilloscope, two oscillators that are almost but not exactly in ratio produce exactly this slow drift, and its speed measures the frequency mismatch.

## Algorithm

1. Choose A, B, a, b and δ.
2. Sample t over one period, 0 to 2π (the app uses 720 samples), and compute x = A sin(a t + δπ), y = B sin(b t).
3. Scale and center the points into a square plot, flipping y for the screen; draw them as a faint closed path.
4. Animate: advance t each frame and draw the path from 0 to t brightly, with the moving point at its tip.
5. Mark the x value on a track above the plot and the y value on a track to its left, so you can see each oscillator separately.
6. If drift is on, add a small amount to δ every frame (wrapping at 2) and redraw.

```
for j in 0..=SAMPLES:
    t = 2 pi * j / SAMPLES
    vertex(A * sin(a*t + delta*pi), B * sin(b*t))
```

## Complexity

| Task | Time | Space |
| --- | --- | --- |
| one point | O(1): two sines | O(1) |
| full figure with S samples | O(S) | O(S) for the vertex list |
| per frame, with drift on | O(S): the figure changes, so it is resampled | O(S) |

The number of samples needed grows with max(a, b): every lobe needs enough points to look smooth.

## Parameters in the app

- **A** and **B**: horizontal and vertical amplitudes, 0.1 to 1. Unequal values stretch the figure.
- **a** and **b**: the two frequencies, 1 to 9. Their ratio is shown large in the corner as a:b.
- **δ**: phase offset of the horizontal wave, 0 to 2 in units of π.
- **Speed**: how fast the moving point travels along the curve.
- **Drift** (checkbox and rate): when on, δ increases by the chosen rate, 0.01 to 0.3 π per second, so the figure morphs and seems to rotate.
- **Play / Pause** and **Restart**: stop the point, or send it back to t = 0.
- The colored dots on the top and left tracks show x(t) and y(t) alone; the dashed lines show how the two combine into the point.

## Numerical notes and pitfalls

- **Sampling.** Too few samples turn high-frequency figures (such as 9:8) into jagged polygons. Use at least about 40 samples per lobe.
- **Closed only for rational ratios.** With an irrational ratio, or a non-integer frequency, the curve never closes and gradually fills the rectangle. Plotting one period of 2π then shows a gap; the app keeps a and b whole numbers to avoid this.
- **Common factors.** a = 4, b = 2 is the 2:1 figure drawn twice. Divide by gcd(a, b) to get the shortest period.
- **Phase conventions differ.** Some texts put the phase on y, use cosine for one axis, or measure δ in radians instead of multiples of π. The pictures match after a shift, but the numbers do not.
- **Degenerate phases.** For a = b with δ = 0 or 1, the ellipse collapses to a line; with other ratios some phases give a curve that retraces itself, and a drawn "trail" can appear to stop moving.
- **Accumulating t.** Adding a time step to t forever slowly loses precision; wrap t at 2π (or at the period), as the app does.

## Applications

- Oscilloscope X-Y mode: comparing frequencies and measuring phase shift between two signals, and testing for audio stereo phase problems.
- Audio phase meters (goniometers) in mixing and mastering.
- Vibration and rotor analysis: orbit plots of shaft motion.
- Laser shows and scanning: galvanometer mirrors driven by two sine waves.
- Graphic design: the ABC (Australian Broadcasting Corporation) logo is a Lissajous figure with a 1:3 ratio.
- Teaching harmonic motion, resonance and phase.

## History

Nathaniel Bowditch studied these curves in 1815 using a compound pendulum, so they are also called **Bowditch curves**. Jules Antoine Lissajous investigated them systematically in 1857, reflecting a light beam off mirrors attached to two vibrating tuning forks at right angles and projecting the figure on a wall; he used the patterns to compare tuning-fork frequencies precisely. Harmonographs, Victorian drawing machines with two or more pendulums, made decorative versions. In the 20th century the cathode-ray oscilloscope made Lissajous figures a standard laboratory tool.

## References

- Wikipedia, "Lissajous curve".
- Jules Antoine Lissajous, "Mémoire sur l'étude optique des mouvements vibratoires", Annales de chimie et de physique, 1857.
- Anthony Ashton, "Harmonograph: A Visual Guide to the Mathematics of Music", Wooden Books, 2003.
- Eric W. Weisstein, "Lissajous Curve", MathWorld.
