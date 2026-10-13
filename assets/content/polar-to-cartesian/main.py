"""Polar to Cartesian: x = r cos(theta), y = r sin(theta).

Left: a point given by a radius r and an angle theta, with the x and y components
it produces drawn along the axes. Right: the rose curve r = cos(k theta), drawn one
angle at a time. Both panels share the same theta, which sweeps around the circle.
Click or drag on the left panel to place the point; press space to play or pause.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

TAU = 2 * np.pi


def to_cartesian(r, theta):
    """(r, theta) -> (x, y), theta in radians, counterclockwise from the +x axis."""
    return r * np.cos(theta), r * np.sin(theta)


def to_polar(x, y):
    """(x, y) -> (r, theta) with theta in [0, 2 pi)."""
    return np.hypot(x, y), np.arctan2(y, x) % TAU


def rose(theta, k):
    """Radius of the rose r = cos(k theta); a negative r lands on the opposite side."""
    return np.cos(k * theta)


K = 5                # rose parameter: odd k gives k petals, even k gives 2k
STEP = 0.02          # radians of theta per frame
state = {"r": 0.8, "theta": 0.75, "playing": True}

fig, (ax_point, ax_rose) = plt.subplots(1, 2, figsize=(11, 5.8))
fig.suptitle("x = r cos(theta),   y = r sin(theta)")


def draw_grid(ax, title):
    """Polar grid: rings of equal r, spokes of equal theta every 30 degrees."""
    ax.set_title(title)
    ax.set_xlim(-1.25, 1.25)
    ax.set_ylim(-1.25, 1.25)
    ax.set_aspect("equal")
    ax.axis("off")
    a = np.linspace(0, TAU, 200)
    for ring in (0.25, 0.5, 0.75, 1.0):
        ax.plot(*to_cartesian(ring, a), color="0.88", lw=0.8)
    for spoke in range(12):
        x, y = to_cartesian(1.0, spoke * np.pi / 6)
        ax.plot([0, x], [0, y], color="0.88", lw=0.8)
        if spoke % 3 == 0:
            ax.text(*to_cartesian(1.13, spoke * np.pi / 6), f"{spoke * 30}",
                    ha="center", va="center", color="0.5")
    ax.plot([-1.05, 1.05], [0, 0], color="0.6", lw=0.8)
    ax.plot([0, 0], [-1.05, 1.05], color="0.6", lw=0.8)


def make_artists(ax):
    """The pieces drawn for one point: components, drop lines, radius, arc, readout."""
    return {
        "x_bar": ax.plot([], [], color="#d0453a", lw=4, solid_capstyle="butt")[0],
        "y_bar": ax.plot([], [], color="#2f7d4f", lw=4, solid_capstyle="butt")[0],
        "x_drop": ax.plot([], [], color="#d0453a", lw=1, ls="--")[0],
        "y_drop": ax.plot([], [], color="#2f7d4f", lw=1, ls="--")[0],
        "radius": ax.plot([], [], color="#2346a0", lw=2)[0],
        "arc": ax.plot([], [], color="0.4", lw=1.5)[0],
        "point": ax.plot([], [], "o", color="#2346a0", ms=10, mec="white", mew=2)[0],
        "label": ax.text(-1.22, -1.22, "", family="monospace", va="bottom"),
    }


def show(arts, r, theta):
    """Place one point at (r, theta) and draw how it becomes (x, y)."""
    x, y = to_cartesian(r, theta)
    arts["x_bar"].set_data([0, x], [0, 0])
    arts["y_bar"].set_data([0, 0], [0, y])
    arts["x_drop"].set_data([x, x], [y, 0])
    arts["y_drop"].set_data([x, 0], [y, y])
    arts["radius"].set_data([0, x], [0, y])
    arts["arc"].set_data(*to_cartesian(0.16, np.linspace(0, theta, 40)))
    arts["point"].set_data([x], [y])
    arts["label"].set_text(f"r = {r:+.2f}  theta = {np.degrees(theta):3.0f} deg\n"
                           f"x = r cos(theta) = {x:+.2f}\n"
                           f"y = r sin(theta) = {y:+.2f}")


draw_grid(ax_point, "Point: click or drag to place it")
draw_grid(ax_rose, f"Rose curve r = cos({K} theta)")
full = np.linspace(0, TAU, 721)
ax_rose.plot(*to_cartesian(rose(full, K), full), color="0.7", lw=1, ls=":")
(trail,) = ax_rose.plot([], [], color="#2346a0", lw=2.2)
point_arts = make_artists(ax_point)
rose_arts = make_artists(ax_rose)


def on_mouse(event):
    """Convert the pointer back to polar form (the inverse formula) and pause."""
    if event.inaxes is not ax_point or event.button != 1:
        return
    r, theta = to_polar(event.xdata, event.ydata)
    state.update(r=min(r, 1.0), theta=theta, playing=False)


def on_key(event):
    if event.key == " ":
        state["playing"] = not state["playing"]


fig.canvas.mpl_connect("button_press_event", on_mouse)
fig.canvas.mpl_connect("motion_notify_event", on_mouse)
fig.canvas.mpl_connect("key_press_event", on_key)


def update(_frame):
    if state["playing"]:
        state["theta"] = (state["theta"] + STEP) % TAU
    theta = state["theta"]
    show(point_arts, state["r"], theta)
    show(rose_arts, rose(theta, K), theta)
    swept = np.linspace(0, theta, max(2, int(theta / TAU * 720)))
    trail.set_data(*to_cartesian(rose(swept, K), swept))
    return (trail, *point_arts.values(), *rose_arts.values())


anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.tight_layout()
plt.show()
