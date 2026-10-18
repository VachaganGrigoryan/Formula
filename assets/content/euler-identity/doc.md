# Euler's formula

## Overview

Euler's formula e^(iθ) = cos θ + i sin θ says that raising e to an imaginary power gives a point on the unit circle, at angle θ. At θ = π it gives Euler's identity, e^(iπ) + 1 = 0, which links the five constants 0, 1, e, i and π.

The tool shows *why*: it adds up the Taylor series of e^x with x = iθ term by term. The terms are drawn as arrows head to tail; each turns a quarter turn from the one before, and the partial sums spiral in to the point on the circle.

## Formula

```math
e^(iθ) = cos θ + i sin θ
e^(iθ) = Σ (iθ)^k / k!,   k = 0, 1, 2, …
e^(iπ) + 1 = 0
```

| Symbol | Meaning |
| --- | --- |
| θ | the angle in radians, and the length of the arc from 1 to the point |
| i | the imaginary unit, i² = −1; multiplying by i turns a point a quarter turn |
| k | the index of a term; term k is (iθ)^k / k! |
| n | how many terms after the first are added up in the app |

## Intuition

Multiplying by i rotates a point by 90°. So the terms of the series point in the directions 1, i, −1, −i, 1, … in turn: right, up, left, down. Their lengths are θ^k / k!, which first grow (while k < θ) and then shrink very fast because of the factorial. The arrows therefore make a squared-off spiral that overshoots, comes back, and closes in on a point. That point is cos θ + i sin θ, because the right-pointing and left-pointing terms add up to the series of cos θ and the up and down terms to the series of sin θ.

## Algorithm

1. Start with the term 1 and the sum 1.
2. For k = 1 … n, multiply the term by iθ / k: (re, im) becomes (−im·θ/k, re·θ/k). Add it to the sum.
3. Draw each partial sum as the head of an arrow starting at the previous one.

Forming each term from the previous one avoids computing θ^k and k! separately, which overflow quickly and lose precision.

## Complexity

O(n) operations for n terms. To be within ε of the limit you need about the n where θ^(n+1)/(n+1)! < ε: for θ up to 2π, n = 30 gives an error below 10^−9.

## Parameters in the app

- **θ**: the angle, from 0 to 2π. *Turn θ* animates it; *θ = π* jumps to Euler's identity.
- **terms after 1**: how many terms after the first are added, from 0 to 30.
- **Stage**: the unit circle, the arrows (real terms in one color, imaginary terms in the other), the target point e^(iθ), and the remaining distance between the partial sum and the point.
- **Derive tab**: the derivation from the Taylor series, and cos θ and sin θ worked out for the current θ.

## Numerical notes and pitfalls

- Summing the series is a fine demonstration but a poor way to compute sines for large θ: the terms grow to about e^θ before shrinking, and adding large terms of alternating sign loses digits. Libraries reduce θ into [−π/4, π/4] first.
- In floating point, e^(iπ) + 1 is not exactly 0: sin(π) evaluates to about 1.2 × 10^−16 because π itself is rounded.
- The app draws arrows shorter than a few points without heads, so the tail of the spiral stays readable.

## Applications

- Phasors in AC circuit analysis: a sinusoid A cos(ωt + φ) is the real part of A e^(i(ωt + φ)).
- The Fourier transform and the FFT decompose signals into e^(iωt) components.
- Rotations in 2D graphics are multiplication by e^(iθ); quaternions extend the idea to 3D.
- Quantum mechanics writes wave functions and time evolution with complex exponentials.

## History

Roger Cotes found an equivalent logarithmic form in 1714. Leonhard Euler published e^(ix) = cos x + i sin x in his *Introductio in analysin infinitorum* (1748), deriving it from the power series exactly as the tool does. The identity e^(iπ) + 1 = 0 follows at once, though Euler never wrote it in that form. Caspar Wessel (1799) and Jean-Robert Argand (1806) introduced the picture of complex numbers as points in the plane, which turns the formula into a statement about rotation.

## References

- Leonhard Euler, *Introductio in analysin infinitorum*, 1748, chapter 8.
- Tristan Needham, *Visual Complex Analysis*, Oxford University Press, 1997, chapter 1.
- Paul J. Nahin, *Dr. Euler's Fabulous Formula*, Princeton University Press, 2006.
