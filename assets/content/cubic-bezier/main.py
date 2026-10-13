"""Cubic Bezier curve: B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3.

The curve is drawn up to the current t, which sweeps 0 -> 1 -> 0. The colored lines
are de Casteljau's construction (lerps of lerps) that lands on the same point, and
the bars below show the four Bernstein weights at t. Drag the four points.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation


def bernstein(t):
    """The four weights (1-t)^3, 3(1-t)^2 t, 3(1-t) t^2, t^3; they always sum to 1."""
    u = 1 - t
    return np.array([u ** 3, 3 * u ** 2 * t, 3 * u * t ** 2, t ** 3])


def bezier(p, t):
    """B(t) for an array of t: the Bernstein-weighted sum of the 4 control points."""
    return bernstein(np.atleast_1d(t)).T @ p


def lerp(a, b, t):
    return (1 - t) * a + t * b


def de_casteljau(p, t):
    """Four points -> three -> two -> one, by repeated lerps."""
    a = lerp(p[:-1], p[1:], t)
    b = lerp(a[:-1], a[1:], t)
    return a, b, lerp(b[0], b[1], t)


def ping_pong(s):
    """Map any real s onto 0..1..0 at constant speed (a triangle wave)."""
    return 1 - abs(1 - s % 2)


points = np.array([[0.08, 0.15], [0.22, 0.92], [0.70, 0.95], [0.92, 0.25]])
state = {"drag": None}

fig, (ax, ax_w) = plt.subplots(2, 1, figsize=(7, 8), gridspec_kw={"height_ratios": [4, 1]})
fig.suptitle("B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3")
ax.set_xlim(0, 1)
ax.set_ylim(0, 1.05)
ax.set_aspect("equal")
ax.set_xticks([])
ax.set_yticks([])

(polygon,) = ax.plot([], [], "--", color="0.6", lw=1)
(full,) = ax.plot([], [], color="#2346a0", lw=1.5, alpha=0.3)
(drawn,) = ax.plot([], [], color="#2346a0", lw=3)
(level1,) = ax.plot([], [], "o-", color="#d0453a", lw=1.5, ms=5)
(level2,) = ax.plot([], [], "o-", color="#2f7d4f", lw=1.5, ms=5)
(handles,) = ax.plot([], [], "o", ms=12, mfc="white", mec="0.2", mew=1.5)
(tip,) = ax.plot([], [], "o", color="#2346a0", ms=11, mec="white", mew=2)
labels = [ax.text(0, 0, f"P{i}", ha="center", color="0.3") for i in range(4)]
t_text = ax.text(0.98, 0.02, "", ha="right", fontsize=14, color="#2346a0")

# The Bernstein weights as four bars.
names = ["(1-t)^3", "3(1-t)^2 t", "3(1-t) t^2", "t^3"]
bars = ax_w.bar(range(4), [0, 0, 0, 0], color="#2346a0", width=0.6)
ax_w.set_ylim(0, 1)
ax_w.set_xticks(range(4), [f"P{i}: {n}" for i, n in enumerate(names)])
ax_w.set_ylabel("weight")


def update(frame):
    t = ping_pong(0.35 + frame / 120)
    ts = np.linspace(0, 1, 121)
    polygon.set_data(*points.T)
    full.set_data(*bezier(points, ts).T)
    drawn.set_data(*bezier(points, ts[ts <= t]).T)
    a, b, c = de_casteljau(points, t)
    level1.set_data(*a.T)
    level2.set_data(*b.T)
    tip.set_data([c[0]], [c[1]])
    handles.set_data(*points.T)
    for label, p in zip(labels, points):
        label.set_position((p[0], p[1] + 0.035))
    for bar, w in zip(bars, bernstein(t)):
        bar.set_height(w)
    t_text.set_text(f"t = {t:.2f}")
    return (polygon, full, drawn, level1, level2, handles, tip, t_text, *labels, *bars)


def on_press(event):
    if event.inaxes is ax:
        d = np.hypot(points[:, 0] - event.xdata, points[:, 1] - event.ydata)
        state["drag"] = int(np.argmin(d)) if d.min() < 0.05 else None


def on_move(event):
    if state["drag"] is not None and event.inaxes is ax:
        points[state["drag"]] = np.clip([event.xdata, event.ydata], 0, 1)


def on_release(_event):
    state["drag"] = None


fig.canvas.mpl_connect("button_press_event", on_press)
fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_release_event", on_release)

anim = FuncAnimation(fig, update, interval=25, cache_frame_data=False)
plt.tight_layout()
plt.show()
