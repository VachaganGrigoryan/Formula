# Lorenz attractor

## Overview

The Lorenz system is three coupled ordinary differential equations for a point `(x, y, z)` moving in space. For the classic parameters its path never settles down and never repeats: it loops around one of two lobes, flips to the other at seemingly random moments, and traces out the butterfly-shaped **Lorenz attractor**. It was the first clear demonstration of **deterministic chaos**: a system with no randomness at all whose long-term behavior is still unpredictable.

The key property is sensitive dependence on initial conditions. Two paths that start 0.00001 apart stay together for a while, then separate exponentially fast until they are on different lobes. This is the reason long-range weather forecasts fail, and the origin of the phrase "the butterfly effect".

## Formula

```math
dx/dt = σ (y − x)
dy/dt = x (ρ − z) − y
dz/dt = x y − β z
σ = 10,   ρ = 28,   β = 8/3
```

| Symbol | Meaning |
| --- | --- |
| `x` | intensity of the convective rolling motion |
| `y` | temperature difference between rising and falling currents |
| `z` | deviation of the vertical temperature profile from linear |
| σ | Prandtl number: viscosity relative to heat diffusion |
| ρ | Rayleigh number, scaled: how strongly the layer is heated |
| β | geometry factor of the convection cells |

The state is advanced numerically with the fourth-order Runge-Kutta method (RK4) and step `h = 0.004`:

```math
k_{1} = f(s),   k_{2} = f(s + h k_{1} / 2),   k_{3} = f(s + h k_{2} / 2),   k_{4} = f(s + h k_{3})
s_{n+1} = s_{n} + h (k_{1} + 2 k_{2} + 2 k_{3} + k_{4}) / 6
```

## Intuition

Each equation has a simple role. The first pulls `x` toward `y` at rate σ. The second makes `y` grow with `x` as long as `z` is below ρ, the heating threshold. The third makes `z` grow when `x` and `y` agree in sign and decay at rate β otherwise. Together they describe a fluid layer heated from below: warm fluid rises, cools, sinks, and the direction of rolling can reverse.

There are three fixed points where all derivatives vanish: the origin and two lobe centers C± = (±√(β(ρ − 1)), ±√(β(ρ − 1)), ρ − 1). For ρ below 1 everything falls into the origin. Between 1 and the Hopf value ρ_H = σ(σ + β + 3) / (σ − β − 1), about 24.74 for the classic values, paths spiral into C+ or C−. Above it both centers repel: a path spirals outward around one center until it is flung across to the other, and the process repeats forever.

The flow contracts volume at the constant rate σ + 1 + β per unit time, so any blob of initial conditions shrinks to zero volume, yet stays bounded and keeps stretching. The result is a fractal attractor of dimension about 2.06. The stretching is measured by the largest Lyapunov exponent, about 0.906: the gap between two nearby paths multiplies by roughly e every 1.1 time units, a straight ramp on a log plot.

## Algorithm

1. Start one path at `(-8, 8, 27)`, near the attractor, and a second one 0.00001 away in `x`.
2. Each frame, advance both paths by a few RK4 steps (step `h = 0.004`, speed time units per second).
3. Append each new state to a fixed-length trail and drop the oldest point.
4. Project to the screen: rotate about the vertical `z` axis by the yaw angle (which slowly increases), tilt by the pitch, then drop the depth coordinate.
5. Draw each trail as a polyline that fades toward its tail, and plot log10 of the distance between the two paths against time.

```
def rk4(s, h):
    k1 = f(s)
    k2 = f(s + h/2 * k1)
    k3 = f(s + h/2 * k2)
    k4 = f(s + h * k3)
    return s + h/6 * (k1 + 2*k2 + 2*k3 + k4)

x1 = x*cos(yaw) - y*sin(yaw);  depth = x*sin(yaw) + y*cos(yaw)
screen = (x1, (z - z0)*cos(pitch) - depth*sin(pitch))
```

## Complexity

| Quantity | Cost |
| --- | --- |
| one RK4 step | 4 evaluations of f, O(1) |
| simulate time T | O(T / h) steps |
| trail of L points, P paths | O(P · L) memory and drawing per frame |
| error per unit time | O(h^4) globally (RK4 is fourth order) |

## Parameters in the app

- **σ**, **ρ**, **β**: the three parameters. Lower ρ below about 24.7 and the paths settle on a lobe center; below 1 they fall into the origin.
- **Speed**: model time units simulated per second of wall time.
- **Trail**: how many integration steps each path keeps on screen.
- **Paths**: 1, 2 or 3 trajectories that start 0.00001 apart in `x`.
- **Play / Pause**, **Restart**, **Classic values** (σ = 10, ρ = 28, β = 8/3), **Spin** (slow rotation about the vertical axis).
- Drag to orbit the camera. The crosses mark the fixed points 0, C+ and C−; the inset plots the distance between the first two paths on a log scale; the readout shows `x, y, z` and their derivatives for the first path.

## Numerical notes and pitfalls

- **Use a higher-order method.** Forward Euler with a large step adds energy and distorts the attractor. RK4 at `h = 0.004` is accurate and cheap; adaptive methods (RK45) work too.
- **Chaos defeats accuracy.** Any numerical error, even rounding, grows like the separation of two paths. After a few dozen time units the computed path is not *the* solution from that start, only *a* path on the attractor. The picture of the attractor is still correct (shadowing).
- **Too large a step.** Accuracy drops quickly for steps above a few hundredths, and beyond about `h = 0.12` RK4 is unstable near the origin, whose fastest eigenvalue is about −22.8. A blown-up state turns into infinity or NaN; check for it and reset.
- **Transients.** A start far from the attractor spends time spiraling in; start near it, as the app does, or discard the first few hundred steps.
- **Trail storage.** Keep trails in a ring buffer or deque so dropping the oldest point is O(1).
- **Projection scale.** The attractor grows with ρ, so scale the view with ρ or it leaves the screen.

## Applications

- Weather and climate science: the limit of predictability and ensemble forecasting, which runs many slightly perturbed forecasts.
- Teaching numerical integration: Euler versus Runge-Kutta, step size and stability.
- Physical systems with the same equations: convection loops (thermosyphons), the Malkus water wheel, simple lasers and some electrical dynamos.
- Chaos-based synchronization and secure communication experiments in electronics.
- Generative art, music visualizers and particle effects that need smooth but never-repeating motion.

## History

Edward Lorenz, a meteorologist at MIT, derived the system in 1963 as a drastically truncated model of convection in a fluid layer, building on Barry Saltzman's equations. He noticed the sensitivity by accident in 1961, when restarting a weather simulation from a printout rounded to three decimals produced a completely different forecast. His paper "Deterministic Nonperiodic Flow" (1963) went largely unnoticed until the 1970s. His 1972 talk "Predictability: Does the flap of a butterfly's wings in Brazil set off a tornado in Texas?" gave the butterfly effect its name. In 2002 Warwick Tucker proved with computer-assisted analysis that the attractor really exists and is chaotic, solving one of Smale's problems.

## References

- Edward N. Lorenz, "Deterministic Nonperiodic Flow", Journal of the Atmospheric Sciences 20, 1963.
- Steven H. Strogatz, *Nonlinear Dynamics and Chaos*, chapter 9, "Lorenz Equations".
- James Gleick, *Chaos: Making a New Science*, 1987.
- Warwick Tucker, "A Rigorous ODE Solver and Smale's 14th Problem", Foundations of Computational Mathematics, 2002.
- Wikipedia, "Lorenz system" and "Runge-Kutta methods".
