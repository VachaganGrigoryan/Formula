"""A* search: f(n) = g(n) + h(n), h = Manhattan distance to the goal

A 31 x 19 grid with a start (orange dot) and a goal (green ring). A* is run once and recorded as a list of operations (a cell joins the priority queue, a cell is
settled, a cell joins the final path); the animation then replays that list at a fixed speed. It is
Dijkstra steered by a Manhattan-distance guess, so it settles far fewer cells and finds a path of the
same cost. Mud costs 5 instead of 1.
Click a cell to toggle a wall or mud (press w to switch the brush) and run the search
again, press m for a random maze and r to restart the replay.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import heapq
import random

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
MUD_COLOR = "#5a4a2a"
MUD = 5           # cost of entering a mud cell


def neighbors(walls, r, c):
    """Open 4-neighbours in the order up, right, down, left."""
    for dr, dc in ((-1, 0), (0, 1), (1, 0), (0, -1)):
        nr, nc = r + dr, c + dc
        if 0 <= nr < H and 0 <= nc < W and not walls[nr, nc]:
            yield nr, nc


def heuristic(r, c):
    """Estimated remaining cost: Manhattan distance to the goal."""
    return abs(r - GOAL[0]) + abs(c - GOAL[1])


def record(walls, mud):
    """Run A* and log every step: ('f', cell), ('v', cell), ('p', cell), ('x',)."""
    ops = [("f", START)]
    dist = {START: 0}
    parent = {}
    done = set()
    heap = [(heuristic(*START), heuristic(*START), START)]    # (f, h, cell)
    found = False
    while heap:
        f, h, cell = heapq.heappop(heap)
        if cell in done or f != dist[cell] + h:     # stale entry: skip it
            continue
        done.add(cell)
        ops.append(("v", cell))
        if cell == GOAL:
            found = True
            break
        for n in neighbors(walls, *cell):
            nd = dist[cell] + (MUD if mud[n] else 1)
            if nd < dist.get(n, float("inf")):          # relaxation
                dist[n] = nd
                parent[n] = cell
                ops.append(("f", n))
                heapq.heappush(heap, (nd + heuristic(*n), heuristic(*n), n))
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


def default_mud():
    """A block of mud across the straight route: the detour around it is cheaper."""
    mud = np.zeros((H, W), bool)
    mud[5:14, 12:19] = True
    return mud


state = {"walls": np.zeros((H, W), bool), "mud": default_mud(), "brush": "wall",
         "ops": [], "cursor": 0, "seed": 0}


def restart():
    """Record the search on the current walls and replay it from the start."""
    state["ops"] = record(state["walls"], state["mud"])
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
    img[state["mud"]] = rgb(MUD_COLOR)
    img[state["walls"]] = rgb(WALL)
    image.set_data(img)
    steps = max(int(state["path"].sum()) - 1, 0)
    cost = steps + (MUD - 1) * int(state["mud"][state["path"]].sum())
    if state["no_path"]:
        title.set_text("No path: the goal is walled off")
    else:
        title.set_text(f"A* search  |  visited {int(state['visited'].sum())}"
                       f"  |  path {steps} steps  |  cost {cost}")
    return image, title


def on_click(event):
    if event.inaxes is not ax or event.xdata is None:
        return
    cell = (int(round(event.ydata)), int(round(event.xdata)))
    if 0 <= cell[0] < H and 0 <= cell[1] < W and cell not in (START, GOAL):
        if state["brush"] == "mud":
            if not state["walls"][cell]:
                state["mud"][cell] = not state["mud"][cell]
        else:
            state["walls"][cell] = not state["walls"][cell]
            state["mud"][cell] = False
        restart()


def on_key(event):
    if event.key == "m":
        state["seed"] += 1
        state["walls"] = maze(state["seed"])
        state["mud"] = np.zeros((H, W), bool)
        restart()
    elif event.key == "w":
        state["brush"] = "wall" if state["brush"] == "mud" else "mud"
    elif event.key == "r":
        restart()


restart()
fig.canvas.mpl_connect("button_press_event", on_click)
fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=INTERVAL, cache_frame_data=False)
plt.show()
