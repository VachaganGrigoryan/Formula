"""Merge sort: sort each half recursively, then merge; T(n) = 2 T(n/2) + O(n) = O(n log n).

A shuffled list drawn as bars, sorted one step at a time. The sort is a recursive
Python generator (each call does `yield from` on its halves) that yields every split,
comparison and write, so the animation just pulls the next step each frame.
Red = the two front values being compared, amber = just written, green = placed by the
final merge, faded = outside the current call. The rows under the bars are the open
recursive calls, outermost on top; the active merge shows its two halves in two colors.
Click the plot to shuffle a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection

N = 48                # number of bars
STEPS_PER_FRAME = 3   # sorting steps applied per animation frame
BLUE, FADED, RED, AMBER, GREEN = "#4a7bd0", "#c3d3ef", "#d64541", "#e8a317", "#2e9e4f"
LEFT_HALF, RIGHT_HALF, OUTER = "#1f9fae", "#9b6fd8", "#b0b0b0"


def merge(a, lo, mid, hi):
    """Merge sorted a[lo:mid] and a[mid:hi] back into a[lo:hi], yielding each step."""
    left, right = a[lo:mid], a[mid:hi]          # the halves, copied aside
    final = lo == 0 and hi == len(a)            # the last merge places values for good
    i = j = 0
    for k in range(lo, hi):
        if i < len(left) and j < len(right):
            yield "compare", lo + i, mid + j    # the two front values, where they started
            take_left = left[i] <= right[j]     # <= keeps equal values in order (stable)
        else:
            take_left = j == len(right)         # one half is used up: copy the other
        if take_left:
            a[k], i = left[i], i + 1
        else:
            a[k], j = right[j], j + 1
        yield "write", k
        if final:
            yield "sorted", k


def merge_sort_steps(a, lo=0, hi=None):
    """Sort a[lo:hi] in place, yielding enter/compare/write/sorted/exit steps."""
    hi = len(a) if hi is None else hi
    if hi - lo < 2:
        return
    mid = lo + (hi - lo) // 2
    yield "enter", lo, mid, hi
    yield from merge_sort_steps(a, lo, mid)
    yield from merge_sort_steps(a, mid, hi)
    yield from merge(a, lo, mid, hi)
    yield ("exit",)


class Run:
    """One sort in progress: the list, its step generator, counters and the call stack."""

    def __init__(self, seed=None):
        rng = np.random.default_rng(seed)
        self.a = [int(v) for v in rng.permutation(N) + 1]
        self.steps = merge_sort_steps(self.a)
        self.sorted = [False] * N
        self.stack = []         # open calls (lo, mid, hi), outermost first
        self.step = None
        self.comparisons = self.writes = 0
        self.done = False
        self.hold = 0           # frames spent showing the finished list

    def advance(self):
        self.step = next(self.steps, None)
        kind = self.step[0] if self.step else "done"
        if kind == "done":
            self.done = True
        elif kind == "enter":
            self.stack.append(self.step[1:])
        elif kind == "exit":
            self.stack.pop()
        elif kind == "compare":
            self.comparisons += 1
        elif kind == "write":
            self.writes += 1
        else:
            self.sorted[self.step[1]] = True

    def color(self, i):
        kind, *where = self.step if self.step else ("", None)
        if kind == "compare" and i in where:
            return RED
        if kind in ("write", "sorted") and i == where[0]:
            return AMBER
        if self.sorted[i] or self.done:
            return GREEN
        if self.stack and not self.stack[-1][0] <= i < self.stack[-1][2]:
            return FADED
        return BLUE


run = Run(seed=2)
fig, ax = plt.subplots(figsize=(9, 5.5))
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
rows = ax.add_collection(LineCollection([], linewidths=3, capstyle="butt"))
ax.set_xlim(-1, N)
ax.set_ylim(-10, N * 1.15)
ax.set_xticks([])
ax.set_yticks([])
ax.set_title("Merge sort: split in halves, then merge sorted halves")
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
    # One row per open call; the innermost (bottom) row shows its two halves.
    segments, colors = [], []
    for depth, (lo, mid, hi) in enumerate(run.stack):
        y = -1.5 - 1.3 * depth
        if depth + 1 < len(run.stack):
            segments.append([(lo - 0.4, y), (hi - 0.6, y)])
            colors.append(OUTER)
        else:
            segments += [[(lo - 0.4, y), (mid - 0.6, y)], [(mid - 0.4, y), (hi - 0.6, y)]]
            colors += [LEFT_HALF, RIGHT_HALF]
    rows.set_segments(segments)
    rows.set_color(colors)
    status = "sorted" if run.done else f"depth {len(run.stack)}   click to reshuffle"
    counter.set_text(f"comparisons {run.comparisons:4d}   writes {run.writes:4d}   {status}")
    return (*bars, rows, counter)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, blit=False, cache_frame_data=False)
plt.tight_layout()
plt.show()
