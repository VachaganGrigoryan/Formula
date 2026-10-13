"""Linear interpolation: v(t) = v0 + t * (v1 - v0).

A point slides between two endpoints, a number moves along a number line, and a
color blends between two colors, all driven by the same t that sweeps 0 -> 1 -> 0.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation


def lerp(a, b, t):
    """Blend a and b; works for scalars and numpy arrays (points, colors)."""
    return (1 - t) * np.asarray(a, dtype=float) + t * np.asarray(b, dtype=float)


def ping_pong(s):
    """Map any real s onto 0..1..0 at constant speed (a triangle wave)."""
    return 1 - abs(1 - s % 2)


P0, P1 = np.array([0.1, 0.2]), np.array([0.9, 0.8])  # point endpoints
V0, V1 = 20.0, 80.0                                     # number endpoints
C0, C1 = np.array([0.14, 0.47, 0.43]), np.array([0.66, 0.44, 0.06])  # RGB endpoints

fig, (ax_plane, ax_line, ax_color) = plt.subplots(
    3, 1, figsize=(7, 7), gridspec_kw={"height_ratios": [5, 1, 1]})
fig.suptitle("v(t) = v0 + t (v1 - v0)")

# The plane: the segment, its endpoints, and the moving point.
ax_plane.set_xlim(0, 1)
ax_plane.set_ylim(0, 1)
ax_plane.set_aspect("equal")
ax_plane.plot(*zip(P0, P1), color="0.7", lw=1, ls="--")
ax_plane.plot(*P0, "o", color=C0, ms=10)
ax_plane.plot(*P1, "o", color=C1, ms=10)
(point,) = ax_plane.plot([], [], "o", color="#2346a0", ms=12)
label = ax_plane.text(0.02, 0.95, "", transform=ax_plane.transAxes, va="top",
                      family="monospace")

# The number line.
ax_line.set_xlim(0, 100)
ax_line.set_ylim(0, 1)
ax_line.set_yticks([])
ax_line.axvline(V0, color=C0)
ax_line.axvline(V1, color=C1)
(number,) = ax_line.plot([], [], "v", color="#2346a0", ms=12)

# The color bar: a full gradient, plus a marker at the current t.
ax_color.imshow(lerp(C0, C1, np.linspace(0, 1, 256)[:, None])[None, :, :],
                extent=(0, 1, 0, 1), aspect="auto")
ax_color.set_yticks([])
marker = ax_color.axvline(0, color="white", lw=3)


def update(frame):
    t = ping_pong(frame / 60)
    p = lerp(P0, P1, t)
    v = lerp(V0, V1, t)
    point.set_data([p[0]], [p[1]])
    number.set_data([v], [0.5])
    marker.set_xdata([t, t])
    label.set_text(f"t = {t:.2f}\nv = {v:5.1f}\np = ({p[0]:.2f}, {p[1]:.2f})")
    return point, number, marker, label


anim = FuncAnimation(fig, update, frames=240, interval=25, blit=False)
plt.tight_layout()
plt.show()
