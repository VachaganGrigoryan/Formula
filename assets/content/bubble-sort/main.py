"""Bubble sort: for each pass, swap neighbours a[j], a[j + 1] whenever a[j] > a[j + 1].

A shuffled list drawn as bars, sorted one step at a time. The sort is a Python
generator that yields every comparison and swap, so the animation just pulls the
next step each frame. Blue = unsorted, red = being compared, amber = just swapped,
green = in its final place. Counters track comparisons and swaps.
Click the plot to shuffle a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

N = 32                # number of bars
STEPS_PER_FRAME = 2   # sorting steps applied per animation frame
BLUE, RED, AMBER, GREEN = "#4a7bd0", "#d64541", "#e8a317", "#2e9e4f"


def bubble_sort_steps(a):
    """Sort list a in place, yielding ("compare", i, j), ("swap", i, j) or ("sorted", i)."""
    n = len(a)
    for i in range(n):
        swapped = False
        for j in range(n - i - 1):
            yield "compare", j, j + 1
            if a[j] > a[j + 1]:
                a[j], a[j + 1] = a[j + 1], a[j]
                swapped = True
                yield "swap", j, j + 1
        yield "sorted", n - i - 1       # the largest remaining value has bubbled to the end
        if not swapped:                 # a pass without swaps: the rest is already in order
            for k in range(n - i - 1):
                yield "sorted", k
            return


class Run:
    """One sort in progress: the list, its step generator, counters and the current step."""

    def __init__(self, seed=None):
        rng = np.random.default_rng(seed)
        self.a = [int(v) for v in rng.permutation(N) + 1]
        self.steps = bubble_sort_steps(self.a)
        self.sorted = [False] * N
        self.step = None
        self.comparisons = self.swaps = 0
        self.done = False
        self.hold = 0           # frames spent showing the finished list

    def advance(self):
        """Pull one step from the generator and update the counters."""
        self.step = next(self.steps, None)
        if self.step is None:
            self.done = True
        elif self.step[0] == "compare":
            self.comparisons += 1
        elif self.step[0] == "swap":
            self.swaps += 1
        else:
            self.sorted[self.step[1]] = True

    def color(self, i):
        kind, *where = self.step if self.step else ("", None)
        if kind == "swap" and i in where:
            return AMBER
        if kind == "compare" and i in where:
            return RED
        return GREEN if self.sorted[i] else BLUE


run = Run(seed=7)
fig, ax = plt.subplots(figsize=(8, 5))
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
ax.set_xlim(-1, N)
ax.set_ylim(0, N * 1.15)
ax.set_xticks([])
ax.set_title("Bubble sort: largest values bubble to the right")
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
    counter.set_text(f"comparisons {run.comparisons:4d}   swaps {run.swaps:4d}   {status}")
    return (*bars, counter)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, blit=False, cache_frame_data=False)
plt.tight_layout()
plt.show()
