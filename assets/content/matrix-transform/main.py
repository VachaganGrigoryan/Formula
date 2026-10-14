"""Matrix transformation: x' = a x + b y,  y' = c x + d y.

A 2x2 matrix moves every point of the plane. Its columns are where the basis
vectors i = (1, 0) and j = (0, 1) land: drag the tips of the two arrows and the
whole grid follows. The shaded cell is the image of the unit square; its area is
|det| and it turns red when det < 0 (the plane is mirrored). Dashed lines are the
eigenvectors, the directions the matrix only stretches: A v = lambda v.
Keys 1-7 load a preset and play the morph from the identity; space replays it.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection
from matplotlib.patches import Polygon

C30, S30 = np.cos(np.pi / 6), np.sin(np.pi / 6)
PRESETS = {
    "1": [[1, 0], [0, 1]],          # identity
    "2": [[1.5, 0], [0, 0.6]],      # scale
    "3": [[C30, -S30], [S30, C30]],  # rotate 30 degrees
    "4": [[1, 1], [0, 1]],          # shear
    "5": [[1, 0], [0, -1]],         # reflect in the x axis
    "6": [[1, 0], [0, 0]],          # project onto the x axis
    "7": [[1, 2], [0.5, 1]],        # singular: both columns on one line
}
HOUSE = np.array([[0.2, 0], [0.8, 0], [0.8, 0.6], [1, 0.6], [0.5, 1.05], [0, 0.6], [0.2, 0.6]])
SQUARE = np.array([[0, 0], [1, 0], [1, 1], [0, 1]])
GRID = 8          # grid lines on each side of the origin
LIMIT = 5.0       # matrix entries stay within +-LIMIT
SECONDS = 1.6     # length of the morph from the identity
INTERVAL = 40     # milliseconds per frame


def eigen(m):
    """('real', [(lam, v), ...]) largest first, ('all', lam) or ('complex', (re, im))."""
    h = (m[0, 0] + m[1, 1]) / 2
    # (trace/2)^2 - det, written so it does not cancel catastrophically.
    disc = ((m[0, 0] - m[1, 1]) / 2) ** 2 + m[0, 1] * m[1, 0]
    eps = 1e-7 * max(1.0, np.abs(m).max()) ** 2  # closer roots count as one
    if disc < -eps:
        return "complex", (h, np.sqrt(-disc))
    repeated = disc <= eps
    lams = [h] if repeated else [h + np.sqrt(disc), h - np.sqrt(disc)]
    # A repeated root is only known to about sqrt(eps): treat a smaller
    # A - lam I as zero (a uniform scale), not as a shear.
    cutoff = 4 * np.sqrt(eps) if repeated else 1e-6
    pairs = []
    for lam in lams:
        # An eigenvector is perpendicular to a nonzero row of A - lam I.
        u = np.array([m[0, 1], lam - m[0, 0]])
        w = np.array([lam - m[1, 1], m[1, 0]])
        v = u if np.hypot(*u) >= np.hypot(*w) else w
        if np.hypot(*v) > cutoff:
            pairs.append((lam, v / np.hypot(*v)))
    return ("real", pairs) if pairs else ("all", h)


state = {"m": np.array([[1.5, 0.5], [0.25, 1.0]]), "t": 0.0, "drag": None}

fig, ax = plt.subplots(figsize=(7.5, 7.5))
ax.set_xlim(-3.5, 3.5)
ax.set_ylim(-3.5, 3.5)
ax.set_aspect("equal")
ax.axis("off")
for k in range(-GRID, GRID + 1):
    ax.plot([k, k], [-GRID, GRID], color="0.9", lw=0.8, zorder=0)
    ax.plot([-GRID, GRID], [k, k], color="0.9", lw=0.8, zorder=0)
grid = ax.add_collection(LineCollection([], colors="#6a8fd8", linewidths=0.8, alpha=0.6))
square = ax.add_patch(Polygon(SQUARE, closed=True, alpha=0.3, lw=0))
ax.add_patch(Polygon(HOUSE, closed=True, fill=False, ls="--", color="0.6"))
house = ax.add_patch(Polygon(HOUSE, closed=True, fill=False, ec="#3355aa", lw=2))
eigen_lines = [ax.plot([], [], "--", color="0.25", lw=1.2)[0] for _ in range(2)]
arrows = ax.quiver([0, 0], [0, 0], [1, 0], [0, 1], color=["#e08a1e", "#2a9d5c"],
                   angles="xy", scale_units="xy", scale=1, width=0.012, zorder=5)
tips = [ax.plot([], [], "o", ms=10, color=c, zorder=6)[0] for c in ("#e08a1e", "#2a9d5c")]
readout = ax.text(-3.4, 3.4, "", family="monospace", fontsize=9, va="top")


def shown():
    """The matrix on screen: part way from the identity while the morph plays."""
    s = min(state["t"], 1.0)
    ease = s * s * (3 - 2 * s)
    return np.eye(2) + ease * (state["m"] - np.eye(2))


def update(_frame):
    state["t"] += INTERVAL / 1000 / SECONDS
    m = shown()
    g = GRID
    segs = [[m @ (k, -g), m @ (k, g)] for k in range(-g, g + 1)]
    segs += [[m @ (-g, k), m @ (g, k)] for k in range(-g, g + 1)]
    grid.set_segments(segs)
    det = np.linalg.det(m)
    square.set_xy(SQUARE @ m.T)
    square.set_color("#d64545" if det < 0 else "#6a8fd8")
    house.set_xy(HOUSE @ m.T)
    arrows.set_UVC(m[0, :], m[1, :])
    for k in range(2):
        tips[k].set_data([m[0, k]], [m[1, k]])

    kind, data = eigen(m)
    for line in eigen_lines:
        line.set_data([], [])
    if kind == "real":
        for line, (_lam, v) in zip(eigen_lines, data):
            line.set_data([-9 * v[0], 9 * v[0]], [-9 * v[1], 9 * v[1]])
        text = "lambda = " + ", ".join(f"{lam:.2f}" for lam, _ in data)
    elif kind == "all":
        text = f"lambda = {data:.2f}: every direction is an eigenvector"
    else:
        text = f"lambda = {data[0]:.2f} +- {data[1]:.2f}i: no real eigenvectors"
    # |det| = |col1| |col2| sin(angle): collapsed when the columns are parallel.
    if abs(det) <= 1e-4 * np.hypot(*m[:, 0]) * np.hypot(*m[:, 1]):
        area = "det = 0: the plane collapses"
    else:
        area = f"det = {det:.2f}" + (" (mirrored)" if det < 0 else "")
    readout.set_text(f"[{m[0, 0]:6.2f} {m[0, 1]:6.2f} ]\n[{m[1, 0]:6.2f} {m[1, 1]:6.2f} ]\n"
                     f"{area}\n{text}")
    return grid, square, house, arrows, readout, *tips, *eigen_lines


def on_press(event):
    if event.inaxes is not ax:
        return
    m = shown()
    for k in range(2):
        if np.hypot(event.xdata - m[0, k], event.ydata - m[1, k]) < 0.2:
            state["m"], state["t"], state["drag"] = m, 1.0, k


def on_move(event):
    k = state["drag"]
    if k is not None and event.inaxes is ax:
        column = np.round(np.array([event.xdata, event.ydata]) * 2) / 2  # snap to 0.5
        state["m"][:, k] = np.clip(column, -LIMIT, LIMIT)


def on_release(_event):
    state["drag"] = None


def on_key(event):
    if event.key in PRESETS:
        state["m"] = np.array(PRESETS[event.key], dtype=float)
        state["t"] = 0.0
    elif event.key in (" ", "space"):
        state["t"] = 0.0


fig.canvas.mpl_connect("button_press_event", on_press)
fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_release_event", on_release)
fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=INTERVAL, cache_frame_data=False)
plt.show()
