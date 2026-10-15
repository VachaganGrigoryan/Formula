"""Breadth-first search: dist(n) = dist(u) + 1, cells visited in order of dist.

A 31 x 19 grid with a start (orange dot) and a goal (green ring). BFS is run
once and recorded as a list of operations (a cell joins the queue, a cell is
visited, a cell joins the final path); the animation then replays that list at a
fixed speed. It explores in rings of equal distance, so the first path it finds
is the shortest one in steps. Click a cell to toggle a wall and run the search
again, press m for a random maze and r to restart the replay.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import random
from collections import deque

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

W, H = 31, 19
START, GOAL = (H // 2, 1), (H // 2, W - 2)  # (row, column)
STEPS = 4         # recorded operations replayed per frame
INTERVAL = 40     # milliseconds per frame
PAPER = "#16201c"
WALL = "#d8e0dc"
FRONTIER = "#9b6a2a"
VISITED = "#2c4a6a"
PATH = "#a8c4ff"
ORANGE = "#f0a040"
GREEN = "#4cc38a"


def neighbors(walls, r, c):
    """Open 4-neighbours in the order up, right, down, left."""
    for dr, dc in ((-1, 0), (0, 1), (1, 0), (0, -1)):
        nr, nc = r + dr, c + dc
        if 0 <= nr < H and 0 <= nc < W and not walls[nr, nc]:
            yield nr, nc


def record(walls):
    """Run BFS and log every step: ('f', cell), ('v', cell), ('p', cell), ('x',)."""
    ops = [("f", START)]
    dist = {START: 0}
    parent = {}
    queue = deque([START])
    found = False
    while queue:
        cell = queue.popleft()
        ops.append(("v", cell))
        if cell == GOAL:
            found = True
            break
        for n in neighbors(walls, *cell):
            if n not in dist:           # mark at enqueue, not at dequeue
                dist[n] = dist[cell] + 1
                parent[n] = cell
                ops.append(("f", n))
                queue.append(n)
    if not found:
        return ops + [("x",)]
    path = [GOAL]
    while path[-1] in parent:
        path.append(parent[path[-1]])
    return ops + [("p", cell) for cell in reversed(path)]


def maze(seed):
    """A perfect maze by recursive backtracking on the odd cells."""
    rng = random.Random(seed)
    walls = np.ones((H, W), bool)
    walls[1, 1] = False
    stack, seen = [(1, 1)], {(1, 1)}
    while stack:
        r, c = stack[-1]
        options = [(r + dr, c + dc) for dr, dc in ((-2, 0), (0, 2), (2, 0), (0, -2))
                   if 1 <= r + dr < H - 1 and 1 <= c + dc < W - 1 and (r + dr, c + dc) not in seen]
        if not options:
            stack.pop()
            continue
        nr, nc = rng.choice(options)
        walls[(r + nr) // 2, (c + nc) // 2] = False
        walls[nr, nc] = False
        seen.add((nr, nc))
        stack.append((nr, nc))
    walls[START] = walls[GOAL] = False
    return walls


def rgb(color):
    return np.array([int(color[i:i + 2], 16) for i in (1, 3, 5)], dtype=float) / 255


state = {"walls": np.zeros((H, W), bool), "ops": [], "cursor": 0, "seed": 0}


def restart():
    """Record the search on the current walls and replay it from the start."""
    state["ops"] = record(state["walls"])
    state["cursor"] = 0
    state["visited"] = np.zeros((H, W), bool)
    state["frontier"] = np.zeros((H, W), bool)
    state["path"] = np.zeros((H, W), bool)
    state["no_path"] = False


def apply(op):
    if op[0] == "f":
        state["frontier"][op[1]] = True
    elif op[0] == "v":
        state["frontier"][op[1]] = False
        state["visited"][op[1]] = True
    elif op[0] == "p":
        state["path"][op[1]] = True
    else:
        state["no_path"] = True


fig, ax = plt.subplots(figsize=(9, 5.8))
fig.patch.set_facecolor(PAPER)
ax.axis("off")
image = ax.imshow(np.zeros((H, W, 3)), interpolation="nearest")
ax.plot([START[1]], [START[0]], "o", ms=11, color=ORANGE)
ax.plot([GOAL[1]], [GOAL[0]], "o", ms=11, mfc="none", mec=GREEN, mew=2.5)
title = ax.set_title("", color="white", family="monospace", fontsize=10)


def update(_frame):
    ops = state["ops"]
    for _ in range(STEPS):
        if state["cursor"] < len(ops):
            apply(ops[state["cursor"]])
            state["cursor"] += 1
    img = np.empty((H, W, 3))
    img[:] = rgb(PAPER)
    img[state["visited"]] = rgb(VISITED)
    img[state["frontier"]] = rgb(FRONTIER)
    img[state["path"]] = rgb(PATH)
    img[state["walls"]] = rgb(WALL)
    image.set_data(img)
    steps = max(int(state["path"].sum()) - 1, 0)
    if state["no_path"]:
        title.set_text("No path: the goal is walled off")
    else:
        title.set_text(f"Breadth-first search  |  visited {int(state['visited'].sum())}"
                       f"  |  path {steps} steps")
    return image, title


def on_click(event):
    if event.inaxes is not ax or event.xdata is None:
        return
    cell = (int(round(event.ydata)), int(round(event.xdata)))
    if 0 <= cell[0] < H and 0 <= cell[1] < W and cell not in (START, GOAL):
        state["walls"][cell] = not state["walls"][cell]
        restart()


def on_key(event):
    if event.key == "m":
        state["seed"] += 1
        state["walls"] = maze(state["seed"])
        restart()
    elif event.key == "r":
        restart()


restart()
fig.canvas.mpl_connect("button_press_event", on_click)
fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=INTERVAL, cache_frame_data=False)
plt.show()
