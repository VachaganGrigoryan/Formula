"""Voronoi diagram: V(p_i) = { x : |x - p_i| <= |x - p_j| for all j != i }.

Each seed owns every location closer to it than to any other seed. Cells are computed
exactly: start from the whole box and cut away, for every other seed, the half-plane
beyond the perpendicular bisector. The seeds drift; press R to toggle Lloyd relaxation
(each seed slides toward its cell's centroid), D to toggle drift. Click to add a seed,
right-click near one to remove it.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import PolyCollection
from matplotlib.colors import hsv_to_rgb

W, H = 160.0, 100.0
BOX = np.array([[0, 0], [W, 0], [W, H], [0, H]], dtype=float)


def clip(poly, p, q):
    """Keep the part of a convex polygon closer to p than to q (Sutherland-Hodgman)."""
    m, n = (p + q) / 2, q - p
    side = (poly - m) @ n  # <= 0 on p's side of the bisector
    out = []
    for k in range(len(poly)):
        a, b, sa, sb = poly[k], poly[(k + 1) % len(poly)], side[k], side[(k + 1) % len(poly)]
        if sa <= 0:
            out.append(a)
        if (sa <= 0) != (sb <= 0):  # the edge crosses the bisector
            out.append(a + (b - a) * sa / (sa - sb))
    return np.array(out).reshape(-1, 2)


def cell(seeds, i):
    """The Voronoi cell of seeds[i] inside the box, as a convex polygon."""
    poly = BOX
    for j, q in enumerate(seeds):
        if j != i and not np.array_equal(q, seeds[i]) and len(poly):
            poly = clip(poly, seeds[i], q)
    return poly


def centroid(poly):
    """Center of mass of a polygon (shoelace formula)."""
    x, y = poly[:, 0], poly[:, 1]
    x1, y1 = np.roll(x, -1), np.roll(y, -1)
    cross = x * y1 - x1 * y
    area2 = cross.sum()
    return np.array([((x + x1) * cross).sum(), ((y + y1) * cross).sum()]) / (3 * area2)


rng = np.random.default_rng(11)
seeds = rng.uniform([3, 3], [W - 3, H - 3], (24, 2))
angles = rng.uniform(0, 2 * np.pi, len(seeds))
vel = np.c_[np.cos(angles), np.sin(angles)] * rng.uniform(4, 10, (len(seeds), 1))
hues = list(rng.uniform(0, 1, len(seeds)))
state = {"drift": True, "relax": False}
plt.rcParams["keymap.home"] = ["h", "home"]  # free the R key for "relax"

fig, ax = plt.subplots(figsize=(8, 5.4))
ax.set_xlim(0, W)
ax.set_ylim(0, H)
ax.set_aspect("equal")
ax.set_xticks([])
ax.set_yticks([])
cells = ax.add_collection(PolyCollection([], edgecolors="white", linewidths=1.2))
dots = ax.scatter([], [], s=14, color="#222", zorder=3)
title = ax.set_title("")


def update(_frame):
    global seeds, vel
    dt = 0.04
    if state["drift"]:
        seeds = seeds + vel * dt
        out = (seeds < 1) | (seeds > [W - 1, H - 1])  # bounce off the walls
        vel = np.where(out, -vel, vel)
        seeds = np.clip(seeds, 1, [W - 1, H - 1])
    polys = [cell(seeds, i) for i in range(len(seeds))]
    if state["relax"]:  # Lloyd: move part of the way to each cell's centroid
        targets = np.array([centroid(p) if len(p) > 2 else s for p, s in zip(polys, seeds)])
        seeds = seeds + (targets - seeds) * min(2 * dt, 1)
    cells.set_verts(polys)
    cells.set_facecolor(hsv_to_rgb(np.c_[hues, np.full(len(hues), 0.35), np.full(len(hues), 0.95)]))
    dots.set_offsets(seeds)
    title.set_text(f"{len(seeds)} seeds   drift {'on' if state['drift'] else 'off'}"
                   f"   Lloyd relax {'on' if state['relax'] else 'off'}   (keys D, R)")
    return cells, dots, title


def on_key(event):
    if event.key in ("r", "d"):
        name = "relax" if event.key == "r" else "drift"
        state[name] = not state[name]


def on_click(event):
    global seeds, vel
    if event.inaxes is not ax:
        return
    at = np.array([event.xdata, event.ydata])
    if event.button == 1:
        seeds, vel = np.vstack([seeds, at]), np.vstack([vel, rng.uniform(-8, 8, 2)])
        hues.append(rng.uniform())
    elif event.button == 3 and len(seeds) > 2:
        k = int(np.argmin(np.hypot(*(seeds - at).T)))
        seeds, vel = np.delete(seeds, k, axis=0), np.delete(vel, k, axis=0)
        hues.pop(k)


fig.canvas.mpl_connect("key_press_event", on_key)
fig.canvas.mpl_connect("button_press_event", on_click)
anim = FuncAnimation(fig, update, frames=None, interval=40, cache_frame_data=False)
plt.tight_layout()
plt.show()
