"""Insertion sort: lift key = a[i], shift every larger a[j] (j < i) one step right, drop key in the gap.

A shuffled list drawn as bars, sorted one step at a time. The sort is a Python
generator that yields every lift, comparison, shift and drop, so the animation just
pulls the next step each frame. The lifted key is the purple outline over its hole
(red while it is compared); amber = just written; faded bars are not yet reached;
everything turns green when sorted. Counters track comparisons and writes.
Click the plot to shuffle a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Rectangle

N = 32                # number of bars
STEPS_PER_FRAME = 3   # sorting steps applied per animation frame
BLUE, FADED, RED, AMBER, GREEN, PURPLE = (
    "#4a7bd0", "#c3d3ef", "#d64541", "#e8a317", "#2e9e4f", "#8e5bd6")


def insertion_sort_steps(a):
    """Sort list a in place, yielding (kind, index, key) for kind in lift/compare/shift/drop."""
    for i in range(1, len(a)):
        key = a[i]                      # lift a[i] out: slot i becomes the hole
        yield "lift", i, key
        j = i - 1
        while j >= 0:
            yield "compare", j, key
            if a[j] <= key:             # first value not larger: the key goes right of it
                break
            a[j + 1] = a[j]             # shift right; the hole moves to j
            yield "shift", j, key
            j -= 1
        a[j + 1] = key                  # drop the key into the hole
        yield "drop", j + 1, key


class Run:
    """One sort in progress: the list, its step generator, counters and the lifted key."""

    def __init__(self, seed=None):
        rng = np.random.default_rng(seed)
        self.a = [int(v) for v in rng.permutation(N) + 1]
        self.steps = insertion_sort_steps(self.a)
        self.step = None
        self.hole = self.key = None
        self.boundary = 0       # a[0..boundary] is the sorted prefix being extended
        self.comparisons = self.writes = 0
        self.done = False
        self.hold = 0           # frames spent showing the finished list

    def advance(self):
        """Pull one step from the generator and update counters and the hole."""
        self.step = next(self.steps, None)
        if self.step is None:
            self.done, self.hole = True, None
            return
        kind, j, key = self.step
        if kind == "lift":
            self.hole, self.key, self.boundary = j, key, j
        elif kind == "compare":
            self.comparisons += 1
        elif kind == "shift":
            self.hole = j
            self.writes += 1
        else:
            self.hole = None
            self.writes += 1

    def color(self, i):
        kind, j, _ = self.step if self.step else ("", -1, 0)
        if self.done:
            return GREEN
        if kind == "compare" and i == j:
            return RED
        if (kind == "shift" and i == j + 1) or (kind == "drop" and i == j):
            return AMBER
        return BLUE if i <= self.boundary else FADED


run = Run(seed=4)
fig, ax = plt.subplots(figsize=(8, 5))
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
ghost = ax.add_patch(Rectangle((0, 0), 0.8, 0, lw=2, ec=PURPLE, fc=PURPLE + "40"))
ax.set_xlim(-1, N)
ax.set_ylim(0, N * 1.15)
ax.set_xticks([])
ax.set_title("Insertion sort: shift larger values right, drop the key in the gap")
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
        bar.set_height(0 if i == run.hole else run.a[i])
        bar.set_color(run.color(i))
    # The lifted key hovers over its hole; red while it is being compared.
    ghost.set_visible(run.hole is not None)
    if run.hole is not None:
        edge = RED if run.step[0] == "compare" else PURPLE
        ghost.set_xy((run.hole - 0.4, 0))
        ghost.set_height(run.key)
        ghost.set_edgecolor(edge)
        ghost.set_facecolor(edge + "40")
    status = "sorted" if run.done else "click to reshuffle"
    counter.set_text(f"comparisons {run.comparisons:4d}   writes {run.writes:4d}   {status}")
    return (*bars, ghost, counter)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, blit=False, cache_frame_data=False)
plt.tight_layout()
plt.show()
