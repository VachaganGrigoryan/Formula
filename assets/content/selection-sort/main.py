"""Selection sort: for i = 0 .. n-2, find m = argmin a[i..n-1] and swap a[i] with a[m].

A shuffled list drawn as bars, sorted one step at a time. The sort is a Python
generator that yields every comparison, minimum update and swap, so the animation
just pulls the next step each frame. Blue = unsorted, red = being compared,
purple = smallest value found so far in this scan, amber = just swapped,
green = in its final place. Counters track comparisons and swaps.
Click the plot to shuffle a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

N = 32                # number of bars
STEPS_PER_FRAME = 3   # sorting steps applied per animation frame
BLUE, RED, AMBER, GREEN, PURPLE = "#4a7bd0", "#d64541", "#e8a317", "#2e9e4f", "#8e5bd6"


def selection_sort_steps(a):
    """Sort list a in place, yielding ("compare", m, j), ("min", m), ("swap", i, m), ("sorted", i)."""
    n = len(a)
    for i in range(n):
        m = i                           # index of the smallest value seen in this scan
        yield "min", m
        for j in range(i + 1, n):
            yield "compare", m, j
            if a[j] < a[m]:
                m = j
                yield "min", m
        if m != i:
            a[i], a[m] = a[m], a[i]
            yield "swap", i, m
        yield "sorted", i               # a[0..i] now holds the i + 1 smallest values


class Run:
    """One sort in progress: the list, its step generator, counters and highlights."""

    def __init__(self, seed=None):
        rng = np.random.default_rng(seed)
        self.a = [int(v) for v in rng.permutation(N) + 1]
        self.steps = selection_sort_steps(self.a)
        self.sorted = [False] * N
        self.step = None
        self.minimum = None
        self.comparisons = self.swaps = 0
        self.done = False
        self.hold = 0           # frames spent showing the finished list

    def advance(self):
        """Pull one step from the generator and update counters and highlights."""
        self.step = next(self.steps, None)
        if self.step is None:
            self.done = True
            return
        kind, i = self.step[0], self.step[1]
        if kind == "compare":
            self.comparisons += 1
        elif kind == "swap":
            self.swaps += 1
        elif kind == "min":
            self.minimum = i
        else:                   # "sorted": the scan is over
            self.sorted[i] = True
            self.minimum = None

    def color(self, i):
        kind, *where = self.step if self.step else ("", None)
        if kind == "swap" and i in where:
            return AMBER
        if i == self.minimum:
            return PURPLE
        if kind == "compare" and i in where:
            return RED
        return GREEN if self.sorted[i] else BLUE


run = Run(seed=3)
fig, ax = plt.subplots(figsize=(8, 5))
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
ax.set_xlim(-1, N)
ax.set_ylim(0, N * 1.15)
ax.set_xticks([])
ax.set_title("Selection sort: pick the smallest, swap it to the front")
counter = ax.text(0.01, 0.97, "", transform=ax.transAxes, va="top", family="monospace")


def restart(_event=None):
    global run
    run = Run()


def update(_frame):
    if run.done:
        run.hold += 1
        if run.hold > 60:       # show the sorted list for a moment, then start over
            restart()
    else:
        for _ in range(STEPS_PER_FRAME):
            run.advance()
            if run.done:
                break
    for i, bar in enumerate(bars):
        bar.set_height(run.a[i])
        bar.set_color(run.color(i))
    status = "sorted" if run.done else "click to reshuffle"
    counter.set_text(f"comparisons {run.comparisons:4d}   swaps {run.swaps:3d}   {status}")
    return (*bars, counter)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, blit=False, cache_frame_data=False)
plt.tight_layout()
plt.show()
