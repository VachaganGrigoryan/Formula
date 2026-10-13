"""Euclidean distance: d = sqrt((x2 - x1)^2 + (y2 - y1)^2).

Left: a probe wanders over a scatter plot (move the mouse there to steer it) and
snaps to the nearest point, as a chart tooltip does. The legs dx and dy and the
hypotenuse d form the Pythagorean triangle; no point lies inside the circle of
radius d. Right: K-means, where each point joins its nearest centroid and each
centroid then moves to the mean of its points. Click the right panel to restart
from new centroids.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection
from matplotlib.patches import Circle

W, H = 100.0, 60.0  # data space, drawn with the same scale on both axes
K = 4               # number of clusters
COLORS = np.array(["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b"])


def distance(a, b):
    """Euclidean distance between a and b (coordinates along the last axis)."""
    d = np.asarray(b, dtype=float) - np.asarray(a, dtype=float)
    return np.sqrt(np.sum(d * d, axis=-1))


def nearest(points, at):
    """Index of the point closest to `at`, and its distance."""
    d = distance(points, at)
    i = int(np.argmin(d))
    return i, d[i]


def kmeans_step(points, centroids):
    """One K-means round: assign to the nearest centroid, then move centroids to the means."""
    d = distance(points[:, None, :], centroids[None, :, :])  # (n, k) table of distances
    assign = np.argmin(d, axis=1)
    moved = centroids.copy()
    for c in range(len(centroids)):
        members = points[assign == c]
        if len(members):  # an empty cluster keeps its centroid
            moved[c] = members.mean(axis=0)
    return assign, moved


rng = np.random.default_rng(2)
blobs = np.array([[25, 18], [75, 18], [25, 42], [75, 42]]) + rng.uniform(-6, 6, (4, 2))
points = np.clip(blobs[np.arange(90) % 4] + rng.normal(0, 4.5, (90, 2)), 1, [W - 1, H - 1])
centroids = rng.uniform([5, 5], [W - 5, H - 5], (K, 2))
state = {"mouse": None, "iteration": 0, "converged": False, "hold": 0}

fig, (ax_near, ax_km) = plt.subplots(1, 2, figsize=(12, 4.8))
for ax in (ax_near, ax_km):
    ax.set_xlim(0, W)
    ax.set_ylim(0, H)
    ax.set_aspect("equal")
    ax.grid(alpha=0.3)

# Nearest point: scatter, empty circle, legs, hypotenuse and a tooltip.
ax_near.scatter(points[:, 0], points[:, 1], s=12, color="0.6")
circle = ax_near.add_patch(Circle((0, 0), 1, fc="#2346a0", ec="#2346a0", alpha=0.12))
(leg_x,) = ax_near.plot([], [], color="#1f77b4", lw=2)
(leg_y,) = ax_near.plot([], [], color="#d62728", lw=2)
(hyp,) = ax_near.plot([], [], color="#2346a0", lw=2.5)
(hit,) = ax_near.plot([], [], "o", ms=10, mfc="#2346a0", mec="white", mew=2)
(probe,) = ax_near.plot([], [], "o", color="black", ms=5)
tip = ax_near.text(0, 0, "", fontsize=9, family="monospace",
                   bbox=dict(boxstyle="round", fc="white", ec="0.7"))

# K-means: spokes from each point to its centroid, colored points, centroid markers.
spokes = ax_km.add_collection(LineCollection([], lw=0.8, alpha=0.3))
dots = ax_km.scatter(points[:, 0], points[:, 1], s=16)
marks = ax_km.scatter(centroids[:, 0], centroids[:, 1], s=220, marker="X",
                      c=COLORS[:K], edgecolors="white", linewidths=1.5, zorder=3)


def probe_at(frame):
    """The mouse position if it is over the left plot, else a wandering Lissajous path."""
    if state["mouse"] is not None:
        return state["mouse"]
    s = frame / 40
    return np.array([W / 2 + 40 * np.sin(1.3 * s), H / 2 + 22 * np.sin(2.1 * s + 0.5)])


def kmeans_tick():
    """Advance K-means by one round; after convergence, pause and restart."""
    if state["converged"]:
        state["hold"] += 1
        if state["hold"] > 4:
            restart()
        return
    _, moved = kmeans_step(points, centroids)
    state["converged"] = np.allclose(moved, centroids, atol=1e-4)
    centroids[:] = moved
    state["iteration"] += 1


def restart():
    centroids[:] = rng.uniform([5, 5], [W - 5, H - 5], (K, 2))
    state.update(iteration=0, converged=False, hold=0)


def update(frame):
    a = probe_at(frame)
    i, d = nearest(points, a)
    b = points[i]
    circle.set_center(a)
    circle.set_radius(d)
    leg_x.set_data([a[0], b[0]], [a[1], a[1]])
    leg_y.set_data([b[0], b[0]], [a[1], b[1]])
    hyp.set_data([a[0], b[0]], [a[1], b[1]])
    hit.set_data([b[0]], [b[1]])
    probe.set_data([a[0]], [a[1]])
    # The tooltip sits on the side of the point away from the probe.
    tip.set_position((b[0] + (-2 if a[0] > b[0] else 2), b[1] + (-2 if a[1] > b[1] else 2)))
    tip.set_ha("right" if a[0] > b[0] else "left")
    tip.set_va("top" if a[1] > b[1] else "bottom")
    tip.set_text(f"point #{i}\n({b[0]:.1f}, {b[1]:.1f})\nd = {d:.2f}")
    ax_near.set_title(f"Nearest point: dx = {abs(b[0] - a[0]):.1f}, dy = {abs(b[1] - a[1]):.1f}")

    if frame % 15 == 0:
        kmeans_tick()
    assign = np.argmin(distance(points[:, None, :], centroids[None, :, :]), axis=1)
    dots.set_color(COLORS[assign])
    spokes.set_segments([[p, centroids[c]] for p, c in zip(points, assign)])
    spokes.set_color(COLORS[assign])
    marks.set_offsets(centroids)
    status = "converged" if state["converged"] else "running"
    ax_km.set_title(f"K-means, k = {K}: iteration {state['iteration']} ({status})")
    return circle, leg_x, leg_y, hyp, hit, probe, tip, dots, spokes, marks


def on_move(event):
    if event.inaxes is ax_near and event.xdata is not None:
        state["mouse"] = np.array([event.xdata, event.ydata])


def on_click(event):
    if event.inaxes is ax_km:
        restart()


fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_press_event", on_click)
anim = FuncAnimation(fig, update, frames=None, interval=40, cache_frame_data=False)
plt.tight_layout()
plt.show()
