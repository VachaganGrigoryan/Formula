"""Drawn by circles: Fourier epicycles.

x_N(t) = a0 + sum_{k=1..N} r_k cos(k t - phi_k)      (and the same for y)

A hand-drawn outline (a cat's head) is resampled evenly, and each coordinate is
expanded into a Fourier series. Every harmonic is a circle spinning k times per
turn; chained largest first, the top chain's tip gives x and the left chain's tip
gives y, and where they meet the outline is drawn. Drag the slider to change the
number of terms N and watch the details melt away. Space plays or pauses.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Polygon
from matplotlib.widgets import Slider

TAU = 2 * np.pi
SAMPLES = 256    # points taken along the outline
MAX_TERMS = 80   # highest harmonic computed

# A closed outline, written by hand: a cat's head with two pointed ears (y up).
OUTLINE = np.array([
    [0.00, -0.90], [0.35, -0.82], [0.62, -0.62], [0.78, -0.32], [0.82, 0.00],
    [0.78, 0.30], [0.70, 0.52], [0.82, 1.05], [0.40, 0.72], [0.20, 0.75],
    [0.00, 0.76], [-0.20, 0.75], [-0.40, 0.72], [-0.82, 1.05], [-0.70, 0.52],
    [-0.78, 0.30], [-0.82, 0.00], [-0.78, -0.32], [-0.62, -0.62], [-0.35, -0.82],
])


def resample(poly, m):
    """m points evenly spaced by arc length along the closed polyline."""
    closed = np.vstack([poly, poly[:1]])
    s = np.concatenate([[0], np.cumsum(np.hypot(*np.diff(closed, axis=0).T))])
    at = np.linspace(0, s[-1], m, endpoint=False)
    return np.column_stack([np.interp(at, s, closed[:, 0]), np.interp(at, s, closed[:, 1])])


def normalize(p):
    """Center the bounding box on 0 and scale its longer side to 1."""
    lo, hi = p.min(axis=0), p.max(axis=0)
    return (p - (lo + hi) / 2) / (hi - lo).max()


def fourier(v, terms):
    """DFT of evenly spaced samples v: mean, then radius r_k and phase phi_k for k = 1..terms."""
    m = len(v)
    angle = TAU * np.arange(1, terms + 1)[:, None] * np.arange(m) / m
    a = 2 / m * (v * np.cos(angle)).sum(axis=1)
    b = 2 / m * (v * np.sin(angle)).sum(axis=1)
    return v.mean(), np.hypot(a, b), np.arctan2(b, a)  # a cos kt + b sin kt = r cos(kt - phi)


def partial_sum(series, n, t):
    """mean + sum of the first n harmonics r_k cos(k t - phi_k), for an array of t."""
    mean, r, phi = series
    k = np.arange(1, n + 1)[:, None]
    return mean + (r[:n, None] * np.cos(k * np.atleast_1d(t) - phi[:n, None])).sum(axis=0)


def chain(series, n, t, origin, swap):
    """Arm joints and circle outlines of the first n terms, largest circle first."""
    _, r, phi = series
    order = np.argsort(-r[:n])
    ang = (order + 1) * t - phi[order]
    step = r[order, None] * np.column_stack([np.cos(ang), np.sin(ang)])
    if swap:  # cosine drives the vertical axis, so the tip tracks y
        step = step[:, ::-1]
    joints = np.vstack([origin, origin + np.cumsum(step, axis=0)])
    ring = np.append(np.linspace(0, TAU, 33), np.nan)
    circles = [(c + rr * np.column_stack([np.cos(ring), np.sin(ring)]))
               for c, rr in zip(joints[:-1], r[order])]
    return joints, np.vstack(circles)


points = normalize(resample(OUTLINE, SAMPLES))
series_x = fourier(points[:, 0], MAX_TERMS)
series_y = fourier(points[:, 1], MAX_TERMS)
# The x chain hangs above the drawing, the y chain to its left; both start at the mean.
X_ORIGIN = np.array([series_x[0], 0.8])
Y_ORIGIN = np.array([-0.85, series_y[0]])
state = {"t": 0.0, "n": 1, "target": 40, "grow": 0, "playing": True}

fig, ax = plt.subplots(figsize=(8, 8))
fig.subplots_adjust(bottom=0.1)
ax.set_xlim(-1.5, 0.6)
ax.set_ylim(-0.62, 1.42)
ax.set_aspect("equal")
ax.axis("off")
ax.plot(*np.vstack([points, points[:1]]).T, ":", color="0.6", lw=1)
fill = ax.add_patch(Polygon(np.zeros((1, 2)), color="#2346a0", alpha=0.12, lw=0))
(trail,) = ax.plot([], [], color="#2346a0", lw=2.2)
arts = {}
for name, color in [("x", "#d0453a"), ("y", "#2f7d4f")]:
    arts[name + "_circles"] = ax.plot([], [], color=color, lw=0.8, alpha=0.45)[0]
    arts[name + "_arms"] = ax.plot([], [], color=color, lw=1.2)[0]
    arts[name + "_link"] = ax.plot([], [], "--", color=color, lw=1)[0]
(pen,) = ax.plot([], [], "o", color="black", ms=5)
count = ax.text(0.58, -0.6, "", ha="right", fontsize=22, color="#2346a0")

slider = Slider(fig.add_axes([0.2, 0.03, 0.6, 0.03]), "Terms N", 1, MAX_TERMS,
                valinit=state["target"], valstep=1)
slider.on_changed(lambda v: state.update(target=int(v), n=int(v), grow=None))


def on_key(event):
    if event.key == " ":
        state["playing"] = not state["playing"]


fig.canvas.mpl_connect("key_press_event", on_key)


def update(_frame):
    if state["grow"] is not None:  # ease from 1 term up to the target over 60 frames
        state["grow"] += 1
        u = min(state["grow"] / 60, 1.0)
        state["n"] = round(1 + (1 - (1 - u) ** 3) * (state["target"] - 1))
        if u >= 1:
            state["grow"] = None
    if state["playing"]:
        state["t"] = (state["t"] + TAU / 500) % TAU
    n, t = state["n"], state["t"]

    whole = np.linspace(0, TAU, 400)
    fill.set_xy(np.column_stack([partial_sum(series_x, n, whole), partial_sum(series_y, n, whole)]))
    drawn = np.linspace(0, t, max(2, int(t / TAU * 400)))
    trail.set_data(partial_sum(series_x, n, drawn), partial_sum(series_y, n, drawn))

    x_joints, x_circles = chain(series_x, n, t, X_ORIGIN, swap=False)
    y_joints, y_circles = chain(series_y, n, t, Y_ORIGIN, swap=True)
    tip = np.array([x_joints[-1, 0], y_joints[-1, 1]])
    for name, joints, circles in [("x", x_joints, x_circles), ("y", y_joints, y_circles)]:
        arts[name + "_circles"].set_data(*circles.T)
        arts[name + "_arms"].set_data(*joints.T)
        arts[name + "_link"].set_data(*np.vstack([joints[-1], tip]).T)
    pen.set_data([tip[0]], [tip[1]])
    count.set_text(f"N = {n}")
    return (fill, trail, pen, count, *arts.values())


anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.show()
