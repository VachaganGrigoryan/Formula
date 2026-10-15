"""Kruskal's algorithm: sort edges by weight, keep each one that joins two trees.

14 random points joined by every pair closer than 0.35 (the seed is retried
until the graph is connected). Kruskal is run once and recorded as a list of
operations (an edge is considered, accepted or rejected); the animation then
replays that list. Vertices are colored by their union-find tree. Click the
plot to draw a new set of points.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import random

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

N = 14
RADIUS = 0.35
STEPS = 1         # recorded operations replayed per frame
INTERVAL = 120    # milliseconds per frame
PAPER = "#16201c"
RULE = "#3a4a44"
CONSIDER = "#f0a040"
TREE = "#a8c4ff"
REJECT = "#7a8a84"
INK = "#d8e0dc"
CYCLE = ["#4cc38a", "#f0a040", "#a8c4ff", "#d8e0dc", "#7a8a84", "#2c8a6a"]


def make_graph(seed):
    """Random points and every pair within RADIUS; retry until connected."""
    while True:
        rng = random.Random(seed)
        pts = np.array([[rng.uniform(0.08, 0.92), rng.uniform(0.08, 0.92)] for _ in range(N)])
        edges = []
        for a in range(N):
            for b in range(a + 1, N):
                w = float(np.hypot(*(pts[a] - pts[b]))) * 100
                if w <= RADIUS * 100:
                    edges.append((a, b, w))
        parent = list(range(N))

        def find(x):
            while parent[x] != x:
                x = parent[x]
            return x

        for a, b, _ in edges:
            parent[find(a)] = find(b)
        if len({find(v) for v in range(N)}) == 1:
            return pts, edges
        seed += 1000


def record(edges):
    """Run Kruskal and log every step: ('c', e), ('a', e), ('r', e), ('u', child, root)."""
    order = sorted(range(len(edges)), key=lambda e: (edges[e][2], e))
    parent = list(range(N))
    size = [1] * N

    def find(x):
        while parent[x] != x:
            x = parent[x]
        return x

    ops = []
    taken = 0
    for e in order:
        if taken + 1 >= N:
            break
        ops.append(("c", e))
        ra, rb = find(edges[e][0]), find(edges[e][1])
        if ra == rb:
            ops.append(("r", e))
            continue
        ops.append(("a", e))
        taken += 1
        child, root = (ra, rb) if size[ra] < size[rb] else (rb, ra)
        parent[child] = root
        size[root] += size[child]
        ops.append(("u", child, root))
    return ops


state = {"seed": 7}
fig, ax = plt.subplots(figsize=(7, 7))
fig.patch.set_facecolor(PAPER)
ax.set_facecolor(PAPER)
ax.axis("off")
ax.set_xlim(0, 1)
ax.set_ylim(1, 0)
ax.set_aspect("equal")
title = ax.set_title("", color="white", family="monospace", fontsize=10)
artists = []


def restart():
    """Draw a graph for the current seed and replay Kruskal on it."""
    for a in artists:
        a.remove()
    artists.clear()
    state["pts"], state["edges"] = make_graph(state["seed"])
    state["ops"] = record(state["edges"])
    state["cursor"] = 0
    state["parent"] = list(range(N))
    state["status"] = ["u"] * len(state["edges"])
    state["weight"] = 0.0
    state["accepted"] = 0
    pts = state["pts"]
    state["lines"] = []
    for a, b, _ in state["edges"]:
        (line,) = ax.plot([pts[a, 0], pts[b, 0]], [pts[a, 1], pts[b, 1]], color=RULE, lw=1, zorder=1)
        state["lines"].append(line)
        artists.append(line)
    state["dots"] = ax.scatter(pts[:, 0], pts[:, 1], s=120, c=CYCLE[0], edgecolors=INK, zorder=3)
    artists.append(state["dots"])


def root(x):
    while state["parent"][x] != x:
        x = state["parent"][x]
    return x


def apply(op):
    kind = op[0]
    if kind == "u":
        state["parent"][op[1]] = op[2]
        return
    e = op[1]
    state["status"][e] = kind
    if kind == "a":
        state["weight"] += state["edges"][e][2]
        state["accepted"] += 1


def style(line, status):
    if status == "c":
        line.set_color(CONSIDER)
        line.set_linewidth(3)
        line.set_linestyle("-")
    elif status == "a":
        line.set_color(TREE)
        line.set_linewidth(3)
        line.set_linestyle("-")
    elif status == "r":
        line.set_color(REJECT)
        line.set_linewidth(1.2)
        line.set_linestyle("--")
    else:
        line.set_color(RULE)
        line.set_linewidth(1)
        line.set_linestyle("-")


def update(_frame):
    ops = state["ops"]
    for _ in range(STEPS):
        if state["cursor"] < len(ops):
            apply(ops[state["cursor"]])
            state["cursor"] += 1
    for line, status in zip(state["lines"], state["status"]):
        style(line, status)
    state["dots"].set_facecolor([CYCLE[root(v) % len(CYCLE)] for v in range(N)])
    title.set_text(f"Kruskal's algorithm  |  tree weight {state['weight']:.0f}"
                   f"  |  edges {state['accepted']} / {N - 1}")
    return title,


def on_click(event):
    if event.inaxes is ax:
        state["seed"] += 1
        restart()


restart()
fig.canvas.mpl_connect("button_press_event", on_click)
anim = FuncAnimation(fig, update, frames=None, interval=INTERVAL, cache_frame_data=False)
plt.show()
