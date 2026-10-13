"""Conway's Game of Life (B3/S23): a cell is born with exactly 3 live neighbors and
survives with 2 or 3; every other cell dies or stays dead.

A random soup on a wrap-around grid evolves one generation per frame. Live cells start
bright and settle to cyan; cells that just died glow and cool to deep blue, leaving
fading trails. Click or drag on the grid to draw live cells.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

COLS, ROWS = 96, 58
FADE = 28        # generations a dead cell takes to fade back to the background
NEVER = 9999     # age of a cell that has never been alive


def neighbors(alive):
    """Live-neighbor count of every cell; np.roll wraps the edges (a torus)."""
    a = alive.astype(np.uint8)
    return sum(np.roll(np.roll(a, dy, axis=0), dx, axis=1)
               for dy in (-1, 0, 1) for dx in (-1, 0, 1) if (dx, dy) != (0, 0))


def step(alive):
    """The next generation, computed for every cell at once."""
    n = neighbors(alive)
    return (n == 3) | (alive & (n == 2))


def update_ages(age, was, now):
    """Generations alive (if alive) or since death (if dead)."""
    age = np.where(now & was, age + 1, age)                     # survived
    age = np.where(now != was, 0, age)                          # just born or just died
    dead_before = ~now & ~was & (age != NEVER)
    return np.where(dead_before, np.minimum(age + 1, FADE), age)


def colors(alive, age):
    """Newborn white -> cyan while alive; cyan -> deep blue -> background once dead."""
    bg = np.array([0.03, 0.055, 0.08])
    newborn, adult = np.array([0.94, 1.0, 1.0]), np.array([0.3, 0.85, 0.9])
    ghosts = np.array([[0.17, 0.61, 0.78], [0.12, 0.24, 0.55], [0.08, 0.12, 0.27], bg])
    t_live = np.clip(age / 6, 0, 1)[..., None]
    live = newborn + (adult - newborn) * t_live
    t = np.clip(age / FADE, 0, 1) * (len(ghosts) - 1)
    k = np.minimum(t.astype(int), len(ghosts) - 2)
    dead = ghosts[k] + (ghosts[k + 1] - ghosts[k]) * (t - k)[..., None]
    rgb = np.where(alive[..., None], live, dead)
    rgb[~alive & (age == NEVER)] = bg
    return rgb


rng = np.random.default_rng(1)
state = {"alive": rng.random((ROWS, COLS)) < 0.28, "gen": 0, "drawing": False}
state["age"] = np.where(state["alive"], 0, NEVER)

fig, ax = plt.subplots(figsize=(9, 6))
image = ax.imshow(colors(state["alive"], state["age"]), interpolation="nearest")
ax.set_xticks([])
ax.set_yticks([])
title = ax.set_title("")


def paint(event):
    if event.inaxes is ax and event.xdata is not None:
        x, y = int(round(event.xdata)) % COLS, int(round(event.ydata)) % ROWS
        state["alive"][y, x] = True
        state["age"][y, x] = 0


def on_press(event):
    state["drawing"] = True
    paint(event)


def on_move(event):
    if state["drawing"]:
        paint(event)


def on_release(_event):
    state["drawing"] = False


fig.canvas.mpl_connect("button_press_event", on_press)
fig.canvas.mpl_connect("motion_notify_event", on_move)
fig.canvas.mpl_connect("button_release_event", on_release)


def update(_frame):
    was = state["alive"]
    now = step(was)
    state["age"] = update_ages(state["age"], was, now)
    state["alive"] = now
    state["gen"] += 1
    image.set_data(colors(now, state["age"]))
    title.set_text(f"B3/S23   generation {state['gen']}   {now.sum()} alive   "
                   "(click or drag to draw)")
    return image, title


anim = FuncAnimation(fig, update, frames=10_000, interval=80, blit=False)
plt.tight_layout()
plt.show()
