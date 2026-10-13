"""Springs and repulsion: F_s = -k (d - x0) on every edge, F_r = kr / d^2 between every pair.

A random graph of four clusters relaxes into a readable layout. Edge color shows the
spring's strain (red stretched, blue squeezed). Drag a node with the mouse to pin it;
the arrows show the spring force and the repulsion acting on it.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection

K, REST, KR = 0.04, 60.0, 3000.0     # stiffness, rest length, repulsion strength
GRAVITY, DAMPING, MAX_SPEED = 0.004, 0.85, 12.0


def spring(k, d, rest):
    """Hooke's law along an edge: negative pulls the two ends together."""
    return -k * (d - rest)


def repulsion(kr, d):
    """Coulomb-style repulsion: always pushes apart, falls off as 1 / d^2."""
    return kr / d**2


def forces(pos, edges):
    """Spring and repulsion force on every node (two arrays of shape (n, 2))."""
    springs, (i, j) = np.zeros_like(pos), edges.T
    delta = pos[i] - pos[j]
    d = np.maximum(np.linalg.norm(delta, axis=1), 1e-3)
    f = delta / d[:, None] * spring(K, d, REST)[:, None]
    np.add.at(springs, i, f)
    np.add.at(springs, j, -f)
    delta = pos[:, None, :] - pos[None, :, :]          # all pairs at once
    d = np.maximum(np.linalg.norm(delta, axis=2), 4.0)
    repel = (delta / d[..., None] * repulsion(KR, d)[..., None]).sum(axis=1)
    return springs, repel


def step(pos, vel, edges, pinned=None):
    """One damped integration step; returns the kinetic energy."""
    springs, repel = forces(pos, edges)
    vel[:] = (vel + springs + repel - GRAVITY * pos) * DAMPING
    speed = np.linalg.norm(vel, axis=1, keepdims=True)
    vel *= np.minimum(1.0, MAX_SPEED / np.maximum(speed, 1e-9))   # cap the step
    if pinned is not None:
        vel[pinned] = 0
    pos += vel
    return 0.5 * (vel**2).sum()


def random_graph(rng, clusters=4):
    """Each cluster is a random tree plus a few extra edges; clusters are chained."""
    sizes = rng.integers(5, 10, clusters)
    edges, cluster_of, start = [], [], 0
    for c, size in enumerate(sizes):
        cluster_of += [c] * size
        edges += [(start + i, start + rng.integers(i)) for i in range(1, size)]
        for _ in range(size // 3):
            a, b = start + rng.integers(size, size=2)
            if a != b and (a, b) not in edges and (b, a) not in edges:
                edges.append((a, b))
        if c > 0:
            edges.append((start - sizes[c - 1], start))
        start += size
    pos = rng.uniform([-150, -110], [150, 110], (start, 2))
    return pos, np.array(edges), np.array(cluster_of)


pos, edges, cluster = random_graph(np.random.default_rng(1))
vel = np.zeros_like(pos)
state = {"drag": None, "hover": None, "energy": 0.0}

fig, ax = plt.subplots(figsize=(8, 6))
ax.set_xlim(-300, 300)
ax.set_ylim(-220, 220)
ax.set_aspect("equal")
ax.set_xticks([])
ax.set_yticks([])
ax.set_title("F_s = -k (d - x0)  on edges,   F_r = kr / d^2  between all pairs")
lines = LineCollection([], zorder=1)
ax.add_collection(lines)
dots = ax.scatter(pos[:, 0], pos[:, 1], s=90, zorder=2, edgecolors="white",
                  c=np.array(["#2a9d8f", "#e76f51", "#4a6fa5", "#8d6a9f"])[cluster])
arrows = ax.quiver([0, 0], [0, 0], [0, 0], [0, 0], color=["#d1495b", "#00798c"],
                   angles="xy", scale_units="xy", scale=1, width=0.006, zorder=3)
label = ax.text(0.02, 0.03, "", transform=ax.transAxes, family="monospace")
ax.text(0.02, 0.97, "drag a node   red: spring force   teal: repulsion",
        transform=ax.transAxes, va="top", fontsize=9, color="0.4")


def nearest(event, radius=15):
    if event.inaxes is not ax:
        return None
    d = np.hypot(pos[:, 0] - event.xdata, pos[:, 1] - event.ydata)
    return int(d.argmin()) if d.min() < radius else None


def on_press(event):
    state["drag"] = nearest(event)


def on_move(event):
    state["hover"] = nearest(event)
    if state["drag"] is not None and event.inaxes is ax:
        pos[state["drag"]] = event.xdata, event.ydata


def on_release(_event):
    state["drag"] = None


fig.canvas.mpl_connect("button_press_event", on_press)
fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_release_event", on_release)


def update(_frame):
    for _ in range(2):
        state["energy"] = step(pos, vel, edges, state["drag"])
    np.clip(pos, [-290, -210], [290, 210], out=pos)

    # Strain > 0: stretched (red), < 0: squeezed (blue).
    d = np.linalg.norm(pos[edges[:, 0]] - pos[edges[:, 1]], axis=1)
    strain = np.clip((d - REST) / REST, -1, 1)
    gray, red, blue = np.array([0.6, 0.6, 0.6]), np.array([0.85, 0.2, 0.25]), np.array([0.1, 0.4, 0.85])
    colors = gray + np.abs(strain)[:, None] * np.where(strain[:, None] > 0, red - gray, blue - gray)
    lines.set_segments(pos[edges])
    lines.set_color(colors)
    lines.set_linewidth(1.2 + 2.5 * np.abs(strain))
    dots.set_offsets(pos)

    focus = state["drag"] if state["drag"] is not None else state["hover"]
    if focus is not None:
        springs, repel = forces(pos, edges)
        vecs = np.array([springs[focus], repel[focus]]) * 12
        arrows.set_offsets([pos[focus], pos[focus]])
        arrows.set_UVC(vecs[:, 0], vecs[:, 1])
    else:
        arrows.set_UVC([0, 0], [0, 0])
    status = "settled" if state["energy"] < 0.01 else "moving"
    label.set_text(f"kinetic energy {state['energy']:.3f}  ({status})")
    return lines, dots, arrows, label


anim = FuncAnimation(fig, update, frames=600, interval=30, blit=False)
plt.tight_layout()
plt.show()
