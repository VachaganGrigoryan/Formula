"""Prim's algorithm: grow one tree, always adding the lightest edge leaving it.

14 random points joined by every pair closer than 0.35 (the seed is retried
until the graph is connected). Prim is run once from vertex 0 and recorded as
a list of operations (an edge is considered, accepted or rejected, a vertex
joins the tree); the animation then replays that list. Vertices in the tree
are filled. Click the plot to draw a new set of points.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import heapq
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
    """Run Prim from vertex 0 and log: ('c', e), ('a', e), ('r', e), ('v', vertex)."""
    adj = [[] for _ in range(N)]
    for e, (a, b, _) in enumerate(edges):
        adj[a].append(e)
        adj[b].append(e)
    in_tree = [False] * N
    in_tree[0] = True
    ops = [("v", 0)]
    heap = [(edges[e][2], e) for e in adj[0]]
    heapq.heapify(heap)
    while heap:
        _, e = heapq.heappop(heap)
        ops.append(("c", e))
        a, b, _ = edges[e]
        if in_tree[a] and in_tree[b]:      # stale entry: both ends already in
            ops.append(("r", e))
            continue
        v = b if in_tree[a] else a
        ops.append(("a", e))
        in_tree[v] = True
        ops.append(("v", v))
        for f in adj[v]:
            other = edges[f][0] if edges[f][1] == v else edges[f][1]
            if not in_tree[other]:
                heapq.heappush(heap, (edges[f][2], f))
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
    """Draw a graph for the current seed and replay Prim on it."""
    for a in artists:
        a.remove()
    artists.clear()
    state["pts"], state["edges"] = make_graph(state["seed"])
    state["ops"] = record(state["edges"])
    state["cursor"] = 0
    state["in_tree"] = [False] * N
    state["status"] = ["u"] * len(state["edges"])
    state["weight"] = 0.0
    state["accepted"] = 0
    pts = state["pts"]
    state["lines"] = []
    for a, b, _ in state["edges"]:
        (line,) = ax.plot([pts[a, 0], pts[b, 0]], [pts[a, 1], pts[b, 1]], color=RULE, lw=1, zorder=1)
        state["lines"].append(line)
        artists.append(line)
    state["dots"] = ax.scatter(pts[:, 0], pts[:, 1], s=120, c=PAPER, edgecolors=INK, zorder=3)
    artists.append(state["dots"])


def apply(op):
    kind = op[0]
    if kind == "v":
        state["in_tree"][op[1]] = True
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
    state["dots"].set_facecolor([TREE if t else PAPER for t in state["in_tree"]])
    title.set_text(f"Prim's algorithm  |  tree weight {state['weight']:.0f}"
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
