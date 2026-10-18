"""Pythagorean theorem: a proof by rearrangement.

Four copies of a right triangle with legs a and b sit in the corners of a square of
side a + b, leaving a tilted square of side c uncovered. Three of them then slide,
one after another, until the uncovered part is a square of side a and one of side b.
The big square and the triangles never changed, so c^2 = a^2 + b^2.
Use the sliders to change a and b.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Polygon, Rectangle
from matplotlib.widgets import Slider


def tilted(a, b):
    """The four triangles around the c^2 square: right angle, end of leg a, end of leg b."""
    s = a + b
    return [
        [(0, 0), (a, 0), (0, b)],
        [(s, 0), (s, a), (a, 0)],
        [(s, s), (b, s), (s, a)],
        [(0, s), (0, b), (b, s)],
    ]


def shifts(a, b):
    """How far each triangle slides to uncover a^2 and b^2."""
    return [(0, a), (0, 0), (-b, 0), (a, -b)]


def smoothstep(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)


def triangles(a, b, t):
    tris = [np.array(tri, dtype=float) for tri in tilted(a, b)]
    moves = shifts(a, b)
    for turn, i in enumerate([3, 0, 2]):
        tris[i] = tris[i] + np.array(moves[i]) * smoothstep(t * 3 - turn)
    return tris


fig = plt.figure(figsize=(6.5, 7.5))
ax = fig.add_axes([0.08, 0.2, 0.84, 0.74])
ax.set_aspect("equal")
ax.axis("off")
slider_a = Slider(fig.add_axes([0.2, 0.1, 0.6, 0.03]), "a", 0.5, 6, valinit=3)
slider_b = Slider(fig.add_axes([0.2, 0.05, 0.6, 0.03]), "b", 0.5, 6, valinit=4)
state = {"phase": 0.0}


def cycle(phase, period=10.0):
    """Hold at 0, move to 1, hold, move back."""
    u = (phase % period) / period
    if u < 0.1:
        return 0.0
    if u < 0.45:
        return (u - 0.1) / 0.35
    if u < 0.6:
        return 1.0
    return 1.0 - (u - 0.6) / 0.4


def update(_frame):
    state["phase"] += 0.03
    a, b = slider_a.val, slider_b.val
    s, t = a + b, cycle(state["phase"])
    ax.clear()
    ax.axis("off")
    ax.set_xlim(-0.5, s + 0.5)
    ax.set_ylim(-0.5, s + 0.5)
    ax.add_patch(Rectangle((0, 0), s, s, facecolor="#f3dfb8", edgecolor="black"))
    for tri in triangles(a, b, t):
        ax.add_patch(Polygon(tri, closed=True, facecolor="white", edgecolor="#2346a0"))
    if t < 0.5:
        ax.text(s / 2, s / 2, "c^2", ha="center", va="center", fontsize=18)
    else:
        ax.text(a / 2, a / 2, "a^2", ha="center", va="center", fontsize=18)
        ax.text(a + b / 2, a + b / 2, "b^2", ha="center", va="center", fontsize=18)
    ax.set_title(f"a = {a:.2f}, b = {b:.2f}:  a^2 + b^2 = {a * a + b * b:.2f} = c^2")
    return []


anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.show()
