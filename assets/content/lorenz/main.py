"""Lorenz attractor: dx/dt = sigma (y - x), dy/dt = x (rho - z) - y, dz/dt = x y - beta z.

Two paths start 0.00001 apart and are stepped forward with RK4. The 3D trails are
projected onto the screen (over a faint trace of the whole attractor) while the view
slowly spins; at first the paths overlap, then they split onto different lobes.
The lower plot shows log10 of the distance between them.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection

SIGMA, RHO, BETA = 10.0, 28.0, 8.0 / 3.0     # Lorenz's own values from 1963
H = 0.004                                     # integration step (model time)
STEPS_PER_FRAME = 15
TRAIL = 2500                                  # points kept per path
PITCH = 0.25


def derivative(s):
    """The right-hand side: how fast x, y and z are changing at state s."""
    x, y, z = s
    return np.array([SIGMA * (y - x), x * (RHO - z) - y, x * y - BETA * z])


def rk4(s, h):
    """One fourth-order Runge-Kutta step of size h."""
    k1 = derivative(s)
    k2 = derivative(s + h / 2 * k1)
    k3 = derivative(s + h / 2 * k2)
    k4 = derivative(s + h * k3)
    return s + h / 6 * (k1 + 2 * k2 + 2 * k3 + k4)


def project(points, yaw, pitch=PITCH):
    """Orthographic view: spin about the vertical z axis, then tilt toward the viewer."""
    x, y, z = points[:, 0], points[:, 1], points[:, 2] - 0.85 * RHO
    x1, depth = x * np.cos(yaw) - y * np.sin(yaw), x * np.sin(yaw) + y * np.cos(yaw)
    return np.column_stack([x1, z * np.cos(pitch) - depth * np.sin(pitch)])


def trace(s, steps):
    """Integrate `steps` RK4 steps from s and return every state."""
    out = [s]
    for _ in range(steps):
        out.append(rk4(out[-1], H))
    return np.array(out)


start = np.array([-8.0, 8.0, 27.0])
backdrop = trace(np.array([1.0, 1.0, 20.0]), 8000)[1000:]   # the attractor, traced once
paths = [start, start + [1e-5, 0, 0]]
trails = [[p] for p in paths]
history = {"t": [], "log_d": []}
sim = {"t": 0.0, "yaw": -0.8}   # yaw -0.8: the wings face the viewer

fig, (ax, ax_sep) = plt.subplots(2, 1, figsize=(7, 8), gridspec_kw={"height_ratios": [4, 1]})
fig.patch.set_facecolor("#10161a")
for a in (ax, ax_sep):
    a.set_facecolor("#10161a")
    a.tick_params(colors="0.6")
    for spine in a.spines.values():
        spine.set_color("0.3")
ax.set_xlim(-27, 27)
ax.set_ylim(-27, 27)
ax.set_aspect("equal")
ax.set_xticks([])
ax.set_yticks([])
ax.set_title(r"$\dot{x} = \sigma(y - x),\quad \dot{y} = x(\rho - z) - y,\quad \dot{z} = xy - \beta z$",
             color="0.85")
(ghost,) = ax.plot([], [], color="0.5", lw=0.5, alpha=0.25)
COLORS = [np.array([0.3, 0.8, 0.9]), np.array([1.0, 0.55, 0.3])]
lines = [LineCollection([], lw=1.3) for _ in COLORS]
for lc in lines:
    ax.add_collection(lc)
heads = [ax.plot([], [], "o", color=c, ms=6)[0] for c in COLORS]
readout = ax.text(0.02, 0.98, "", transform=ax.transAxes, va="top", family="monospace",
                  color="0.85", fontsize=9)

ax_sep.set_xlim(0, 40)
ax_sep.set_ylim(-6, 2)
ax_sep.axhline(0, color="0.4", lw=0.8)
ax_sep.set_xlabel("time", color="0.6")
ax_sep.set_ylabel("log10 distance", color="0.6")
(sep_line,) = ax_sep.plot([], [], color=COLORS[1], lw=1.5)


def update(_frame):
    for _ in range(STEPS_PER_FRAME):
        for k in range(len(paths)):
            paths[k] = rk4(paths[k], H)
            trails[k].append(paths[k])
            del trails[k][:-TRAIL]
        sim["t"] += H
    distance = np.linalg.norm(paths[0] - paths[1])
    if sim["t"] <= 40:
        history["t"].append(sim["t"])
        history["log_d"].append(np.log10(max(distance, 1e-12)))
    sim["yaw"] += 0.006   # slow spin

    ghost.set_data(*project(backdrop, sim["yaw"]).T)
    for lc, trail, head, color in zip(lines, trails, heads, COLORS):
        pts = project(np.array(trail), sim["yaw"])
        segments = np.stack([pts[:-1], pts[1:]], axis=1)
        alpha = np.linspace(0.05, 1.0, len(segments))     # fade toward the tail
        lc.set_segments(segments)
        lc.set_color(np.column_stack([np.tile(color, (len(segments), 1)), alpha]))
        head.set_data([pts[-1, 0]], [pts[-1, 1]])
    sep_line.set_data(history["t"], history["log_d"])
    x, y, z = paths[0]
    readout.set_text(f"x = {x:7.2f}\ny = {y:7.2f}\nz = {z:7.2f}\nt = {sim['t']:7.2f}\n"
                     f"gap = {distance:.2e}")
    return (ghost, *lines, *heads, sep_line, readout)


anim = FuncAnimation(fig, update, frames=4000, interval=30, blit=False)
plt.tight_layout()
plt.show()
