# Mandelbrot set

## Overview

The Mandelbrot set is the set of complex numbers `c` for which the simple iteration z -> z² + c, started at `z = 0`, stays bounded forever. Plotting which points of the complex plane belong to it gives the most famous picture in mathematics: a black cardioid fringed with bulbs, filaments and spirals, with tiny copies of the whole set hidden at every scale.

The picture is made with the **escape-time algorithm**. Every pixel is a value of `c`; iterate until `|z|` grows past a bailout radius or an iteration limit runs out, and color the pixel by how many steps the escape took. Points that never escape are drawn black. A smooth, fractional version of the escape count removes the visible color bands and gives the continuous gradients seen in the app.

## Formula

```math
z_{n+1} = z_{n}^{2} + c,   z_{0} = 0
c is in M  if  |z_{n}| ≤ 2 for every n ≥ 0
μ = n + 1 − log_{2}( log_{2} |z_{n}| )
```

| Symbol | Meaning |
| --- | --- |
| `c` | the point being tested, `c = x + iy`, one pixel |
| `z_n` | the n-th iterate; `z_0 = 0` |
| `n` | the first iteration where the squared size of `z_n` exceeds the bailout (256 in the app) |
| μ | smooth (continuous) escape count used for coloring |
| `M` | the Mandelbrot set |

In real arithmetic, with `z = a + bi` and `c = x + yi`, one step is a' = a² − b² + x and b' = 2ab + y. The bailout radius 2 is enough to decide escape; a larger one (16 here, |z|² > 256) makes the smooth count accurate.

## Intuition

Squaring a complex number squares its length and doubles its angle. Once `|z|` is larger than 2 (and at least `|c|`), squaring wins over adding `c`, so `|z|` grows without limit: that is why radius 2 is a safe escape test. Inside the set the two effects balance and the orbit settles into a fixed point, a cycle, or a chaotic but bounded dance.

The large regions have exact descriptions. The main cardioid holds the values of `c` whose orbit converges to a single attracting fixed point; the disc of radius 1/4 centered at −1 holds those that converge to a 2-cycle. Each smaller bulb corresponds to a longer attracting cycle, which is what you see when you hover over a bulb in the app and the orbit hops between the same few points.

Near escape, `|z|` roughly squares each step, so `log |z|` doubles and `log log |z|` grows by `log 2` per step. Subtracting `log₂ log₂ |z_n|` from the integer count measures how far into the step the escape really happened, turning the step function `n` into the continuous μ.

## Algorithm

1. Map the pixel `(i, j)` to c = center + (i − w/2, h/2 − j) · pixel_size.
2. Skip the most expensive pixels: if `c` lies in the main cardioid or the period-2 bulb, it is in the set.
3. Set `z = 0` and iterate z = z² + c up to `max_iter` times.
4. If |z|² exceeds the bailout at step `n`, compute μ and map it through a color gradient.
5. If the limit is reached, color the pixel black.

```
q = (x - 1/4)^2 + y^2
if q * (q + x - 1/4) <= y^2 / 4 or (x + 1)^2 + y^2 <= 1/16: inside
a = b = 0
for n in 0 .. max_iter:
    if a*a + b*b > 256: return n + 1 - log2(log2(sqrt(a*a + b*b)))
    a, b = a*a - b*b + x, 2*a*b + y
return inside
```

## Complexity

| Quantity | Cost |
| --- | --- |
| one pixel | O(max_iter) worst case, usually far fewer |
| one image of P pixels | O(P · max_iter) |
| memory | O(P) for the image, O(1) per pixel |
| zoom depth with 64-bit floats | about 10¹³ before pixels merge |

Every pixel is independent, so the work parallelizes perfectly across threads or GPU shader invocations. Black pixels are the slowest because they always run to the limit; the cardioid test removes most of them in the default view.

## Parameters in the app

- **Max iterations**: the iteration limit (30 to 3000). Deeper zooms need more, or points just outside the set are wrongly drawn black and fine detail disappears.
- **Whole set**, **Seahorse valley**, **Mini Mandelbrot**: jump to the full view, the spiral-filled gap between the cardioid and the period-2 bulb, or a tiny copy of the set on the real axis near −1.75.
- **Hover**: draws the orbit z₀, z₁, z₂, … of the point under the cursor and says whether it escapes and after how many iterations.
- **Click** to zoom in 2x, **right-click** to zoom out, **drag** to pan, and use the **scroll wheel** or a **pinch** to zoom smoothly. The readout shows the current magnification.

## Numerical notes and pitfalls

- **Precision limit.** 64-bit floats have about 16 significant digits. Past a zoom of roughly 10¹³ neighboring pixels map to the same `c` and the picture turns blocky. Deep-zoom renderers use arbitrary precision for one reference orbit and perturbation theory for the rest.
- **Iteration limit.** No finite `max_iter` proves membership; it only says "has not escaped yet". Raise it as you zoom (a common rule grows it with the log of the magnification).
- **Smooth coloring needs a big bailout.** With radius 2 the `log log` correction is inaccurate and bands remain; radius 16 or more makes it smooth. Clamp μ at small values, where the formula can go slightly negative.
- **Banding versus palette.** Map μ through a periodic or square-root scaled gradient so color cycles stay readable at every depth.
- **Pixel center.** Sample at `i + 0.5`, or the image shifts by half a pixel each time you zoom.
- **Aspect ratio.** Use the same scale on both axes or the set appears stretched.

## Applications

- Teaching complex numbers, iteration, chaos and fractal geometry.
- Benchmarks for parallel computing, SIMD and GPU programming, because the work is embarrassingly parallel but unevenly distributed.
- Fractal art, music videos and procedurally generated textures.
- The same escape-time idea renders Julia sets, the Burning Ship fractal and Newton fractals.
- Research in complex dynamics: the boundary is connected, has Hausdorff dimension 2, and the open question of whether it is locally connected (MLC) is still unsolved.

## History

Pierre Fatou and Gaston Julia studied the iteration of rational functions around 1918, long before computers could draw it. Robert Brooks and Peter Matelski produced the first crude picture of the set in 1978. Benoit Mandelbrot, working at IBM, made the first high-quality plots in 1980 and popularized them in *The Fractal Geometry of Nature* (1982). Adrien Douady and John Hubbard proved that the set is connected and named it after Mandelbrot in 1982. A. K. Dewdney's 1985 Scientific American column brought the escape-time program to home computers.

## References

- Benoit B. Mandelbrot, *The Fractal Geometry of Nature*, W. H. Freeman, 1982.
- Heinz-Otto Peitgen and Peter H. Richter, *The Beauty of Fractals*, Springer, 1986.
- Adrien Douady and John H. Hubbard, "Étude dynamique des polynômes complexes", Publications mathématiques d'Orsay, 1984.
- A. K. Dewdney, "Computer Recreations", Scientific American, August 1985.
- Wikipedia, "Mandelbrot set" and "Plotting algorithms for the Mandelbrot set".
