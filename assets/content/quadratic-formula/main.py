"""Quadratic formula: where a x^2 + b x + c = 0 crosses the x axis.

The parabola y = a x^2 + b x + c with its real roots, its vertex and its axis of
symmetry. The discriminant b^2 - 4ac decides how many roots there are: two when it
is positive, one double root when it is zero, none when it is negative.
Use the sliders to change a, b and c.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import Slider

EPS = 1e-12


def roots(a, b, c):
    """Real roots, smaller first, computed without cancellation."""
    if abs(a) < EPS:
        return [] if abs(b) < EPS else [-c / b]
    d = b * b - 4 * a * c
    if abs(d) < EPS:
        return [-b / (2 * a)]
    if d < 0:
        return []
    q = -0.5 * (b + np.copysign(np.sqrt(d), b))
    return sorted([q / a, c / q])


fig = plt.figure(figsize=(7, 7))
ax = fig.add_axes([0.1, 0.25, 0.85, 0.68])
ax.set_xlim(-6, 6)
ax.set_ylim(-5, 5)
ax.grid(True, color="0.9")
ax.axhline(0, color="0.5", lw=1)
ax.axvline(0, color="0.5", lw=1)
xs = np.linspace(-6, 6, 601)
(curve,) = ax.plot([], [], color="#2346a0", lw=2.4)
(axis_line,) = ax.plot([], [], "--", color="#a8700f", lw=1)
(vertex,) = ax.plot([], [], "o", color="#a8700f")
(marks,) = ax.plot([], [], "o", color="#23786e", ms=8)
caption = ax.set_title("")

slider_a = Slider(fig.add_axes([0.2, 0.13, 0.6, 0.03]), "a", -5, 5, valinit=1)
slider_b = Slider(fig.add_axes([0.2, 0.08, 0.6, 0.03]), "b", -5, 5, valinit=-1)
slider_c = Slider(fig.add_axes([0.2, 0.03, 0.6, 0.03]), "c", -5, 5, valinit=-2)


def update(_frame):
    a, b, c = slider_a.val, slider_b.val, slider_c.val
    curve.set_data(xs, a * xs ** 2 + b * xs + c)
    if abs(a) >= EPS:
        vx = -b / (2 * a)
        axis_line.set_data([vx, vx], [-5, 5])
        vertex.set_data([vx], [a * vx ** 2 + b * vx + c])
    else:
        axis_line.set_data([], [])
        vertex.set_data([], [])
    found = roots(a, b, c)
    marks.set_data(found, [0] * len(found))
    d = b * b - 4 * a * c
    caption.set_text(f"discriminant = {d:.2f}: " + ", ".join(f"x = {x:.3f}" for x in found)
                     if found else f"discriminant = {d:.2f}: no real roots")
    return curve, axis_line, vertex, marks, caption


anim = FuncAnimation(fig, update, interval=50, cache_frame_data=False)
plt.show()
