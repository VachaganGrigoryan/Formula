"""2D rotation: x' = x cos(theta) - y sin(theta),  y' = x sin(theta) + y cos(theta).

Top: a letter F spins about the origin. The colored arrows are the rotated axes,
(cos, sin) and (-sin, cos), which are the columns of the rotation matrix. Drag the
point P with the mouse; P' is its image and the dashed arc is the path between them.
Bottom: the same geometry tilts crowded month labels; labels that still collide with
a neighbor are drawn in red.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Polygon


def rotate(points, theta):
    """Rotate points (shape (..., 2)) counter-clockwise about the origin."""
    c, s = np.cos(theta), np.sin(theta)
    x, y = np.asarray(points, dtype=float)[..., 0], np.asarray(points, dtype=float)[..., 1]
    return np.stack([x * c - y * s, x * s + y * c], axis=-1)


def labels_collide(spacing, width, height, angle):
    """Do two equal labels `spacing` apart, tilted by `angle`, overlap?"""
    return spacing * np.cos(angle) < width and spacing * np.sin(angle) < height


F = np.array([[-0.25, -0.5], [-0.05, -0.5], [-0.05, -0.05], [0.2, -0.05], [0.2, 0.12],
              [-0.05, 0.12], [-0.05, 0.32], [0.3, 0.32], [0.3, 0.5], [-0.25, 0.5]])
MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
          "September", "October", "November", "December"]
SALES = [0.42, 0.55, 0.61, 0.48, 0.72, 0.9, 0.84, 0.77, 0.63, 0.58, 0.69, 0.95]
FONT = 10  # label size in points
state = {"p": np.array([0.78, 0.32]), "dragging": False}

fig, (ax, ax_bar) = plt.subplots(2, 1, figsize=(7, 8.5), gridspec_kw={"height_ratios": [3, 1]})
ax.set_xlim(-1.4, 1.4)
ax.set_ylim(-1.3, 1.3)
ax.set_aspect("equal")
ax.axis("off")
ax.add_patch(plt.Circle((0, 0), 1, fill=False, color="0.8"))
ax.plot([-1.15, 1.15], [0, 0], color="0.8", lw=1)
ax.plot([0, 0], [-1.15, 1.15], color="0.8", lw=1)
ax.add_patch(Polygon(F, closed=True, fill=False, ls="--", color="0.6"))
shape = ax.add_patch(Polygon(F, closed=True, fc="#f0b45a55", ec="#c07a10", lw=2))
ax.quiver([0, 0], [0, 0], [1, 0], [0, 1], color=["#1f77b455", "#d6272855"],
          angles="xy", scale_units="xy", scale=1, width=0.006)
axes_arrows = ax.quiver([0, 0], [0, 0], [1, 0], [0, 1], color=["#1f77b4", "#d62728"],
                        angles="xy", scale_units="xy", scale=1, width=0.01)
(arc,) = ax.plot([], [], ":", color="0.4")
(p_dot,) = ax.plot([], [], "o", mfc="white", mec="0.3", ms=11)
(q_dot,) = ax.plot([], [], "o", color="black", ms=8)
p_label = ax.text(0, 0, "P", fontsize=11)
q_label = ax.text(0, 0, "P'", fontsize=11)
readout = ax.text(-1.38, -1.28, "", family="monospace", fontsize=9, va="bottom")

# Bar chart with tiltable month labels.
ax_bar.bar(range(12), SALES, width=0.6, color="#f0b45a")
ax_bar.set_xlim(-0.5, 11.5)
ax_bar.set_ylim(0, 1)
ax_bar.set_xticks([])
ax_bar.set_yticks([])
labels = [ax_bar.text(k, -0.04, m, fontsize=FONT, ha="center", va="top",
                      rotation_mode="anchor") for k, m in enumerate(MONTHS)]
note = ax_bar.set_title("", fontsize=10)
plt.subplots_adjust(left=0.05, right=0.95, top=0.98, bottom=0.17, hspace=0.12)


def update(frame):
    theta = (0.02 * frame + np.pi / 6 + np.pi) % (2 * np.pi) - np.pi
    tilt = np.radians(90 * (1 - abs(1 - (frame / 100) % 2)))  # 0 -> 90 -> 0 degrees

    p = state["p"]
    q = rotate(p, theta)
    shape.set_xy(rotate(F, theta))
    i, j = rotate([1, 0], theta), rotate([0, 1], theta)
    axes_arrows.set_UVC([i[0], j[0]], [i[1], j[1]])
    path = np.array([rotate(p, a) for a in np.linspace(0, theta, 40)])
    arc.set_data(path[:, 0], path[:, 1])
    p_dot.set_data([p[0]], [p[1]])
    q_dot.set_data([q[0]], [q[1]])
    p_label.set_position((p[0] + 0.07, p[1] - 0.1))
    q_label.set_position((q[0] + 0.07, q[1] + 0.05))
    c, s = np.cos(theta), np.sin(theta)
    readout.set_text(f"theta = {np.degrees(theta):4.0f} deg\n"
                     f"x' = {p[0]:.2f}*{c:.2f} - {p[1]:.2f}*{s:.2f} = {q[0]:.2f}\n"
                     f"y' = {p[0]:.2f}*{s:.2f} + {p[1]:.2f}*{c:.2f} = {q[1]:.2f}")

    # Spacing and label sizes in points (an estimate: 0.6 em per character).
    slot = ax_bar.get_position().width * fig.get_figwidth() * 72 / 12
    widths = [0.6 * FONT * len(m) for m in MONTHS]
    hits = [labels_collide(slot, widths[k + 1], 1.2 * FONT, tilt) for k in range(11)]
    for k, label in enumerate(labels):
        bad = (k > 0 and hits[k - 1]) or (k < 11 and hits[k])
        label.set_rotation(np.degrees(tilt))
        label.set_ha("right" if tilt > 0.01 else "center")
        label.set_x(k + 0.15 if tilt > 0.01 else k)
        label.set_color("#d62728" if bad else "black")
    n = sum(hits)
    note.set_text(f"label tilt {np.degrees(tilt):.0f} deg: "
                  + (f"{n} label pairs collide" if n else "no labels collide"))
    return shape, axes_arrows, arc, p_dot, q_dot, p_label, q_label, readout, note, *labels


def on_press(event):
    if event.inaxes is ax and np.hypot(event.xdata - state["p"][0], event.ydata - state["p"][1]) < 0.12:
        state["dragging"] = True


def on_move(event):
    if state["dragging"] and event.inaxes is ax:
        p = np.array([event.xdata, event.ydata])
        state["p"] = p * min(1.0, 1.1 / max(np.hypot(*p), 1e-9))  # keep within radius 1.1


def on_release(_event):
    state["dragging"] = False


fig.canvas.mpl_connect("button_press_event", on_press)
fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_release_event", on_release)
anim = FuncAnimation(fig, update, frames=None, interval=40, cache_frame_data=False)
plt.show()
