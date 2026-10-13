"""Quick sort (Lomuto partition, last element as pivot).

partition(lo, hi): pivot = a[hi]; every a[j] <= pivot is swapped to the front, then
the pivot drops in right behind them, at its final index p, so that
a[lo..p-1] <= a[p] < a[p+1..hi]. Recurse on lo..p-1 and p+1..hi.

The sort is a generator that yields one step at a time (compare, swap, final place)
while it reorders the list in place, so the animation can play it bar by bar.
Blue unsorted, red comparing, amber swapping, pink pivot, green in final place.
The dashed line is the pivot's value, the dark bracket the partition in progress;
bars outside it are faded. Click for a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.colors import to_rgba
from matplotlib.patches import Patch

N = 40
STEPS_PER_FRAME = 2
BLUE, RED, AMBER, GREEN, PINK = "#2346a0", "#d64541", "#a8700f", "#2e9e4f", "#e0569b"
FADED = to_rgba(BLUE, 0.25)


def partition(a, lo, hi):
    """Lomuto partition of a[lo..hi]; yields steps, returns the pivot's final index."""
    yield ("range", lo, hi)
    i = lo                                  # a[lo..i-1] holds the values <= pivot
    for j in range(lo, hi):
        yield ("compare", j, hi)
        if a[j] <= a[hi]:
            if i != j:
                a[i], a[j] = a[j], a[i]
                yield ("swap", i, j)
            i += 1
    if i != hi:                             # drop the pivot in behind the small ones
        a[i], a[hi] = a[hi], a[i]
        yield ("swap", i, hi)
    return i


def quick_sort(a, lo, hi):
    """Sort a[lo..hi] in place, yielding every step."""
    if lo < hi:
        p = yield from partition(a, lo, hi)
        yield ("sorted", p)
        yield from quick_sort(a, lo, p - 1)
        yield from quick_sort(a, p + 1, hi)
    elif lo == hi:
        yield ("sorted", lo)


class Run:
    """One shuffled list being sorted, plus what the picture needs to know."""

    def __init__(self):
        self.a = [int(v) for v in np.random.permutation(N) + 1]
        self.steps = quick_sort(self.a, 0, N - 1)
        self.sorted = [False] * N
        self.hot = {}           # index -> color, for the step just taken
        self.range = None       # (lo, hi) of the partition in progress
        self.pivot = None       # (index, value) of the current pivot
        self.comparisons = self.swaps = 0
        self.done = False
        self.rest = 0           # frames shown since finishing

    def advance(self):
        self.hot = {}
        step = next(self.steps, None)
        if step is None:
            self.done, self.range = True, None
            return
        kind, *idx = step
        if kind == "range":
            lo, hi = idx
            self.range, self.pivot = (lo, hi), (hi, self.a[hi])
        elif kind == "compare":
            self.comparisons += 1
            self.hot = dict.fromkeys(idx, RED)
        elif kind == "swap":
            self.swaps += 1
            self.hot = dict.fromkeys(idx, AMBER)
        elif kind == "sorted":
            self.sorted[idx[0]] = True
            self.pivot = None

    def color(self, k):
        if k in self.hot:
            return self.hot[k]
        if self.pivot and k == self.pivot[0]:
            return PINK
        if self.sorted[k]:
            return GREEN
        if self.range and not self.range[0] <= k <= self.range[1]:
            return FADED
        return BLUE


run = Run()
fig, ax = plt.subplots(figsize=(9, 5.5))
fig.suptitle("Quick sort (Lomuto): a[lo..p-1] <= a[p] < a[p+1..hi]")
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
ax.set_xlim(-1, N)
ax.set_ylim(-3, N + 6)
ax.set_xticks([])
ax.set_yticks([])
(pivot_line,) = ax.plot([], [], "--", color=PINK, lw=1.2)
(bracket,) = ax.plot([], [], color="0.25", lw=3, solid_capstyle="butt")
status = ax.text(0, N + 3, "", family="monospace")
legend = [(BLUE, "unsorted"), (RED, "comparing"), (AMBER, "swapping"),
          (PINK, "pivot"), (GREEN, "in final place")]
fig.legend(handles=[Patch(color=c, label=s) for c, s in legend],
           loc="lower center", ncol=5, frameon=False)


def restart(_event):
    global run
    run = Run()


def update(_frame):
    global run
    if run.done:
        run.rest += 1
        if run.rest > 60:               # admire the result, then start over
            run = Run()
    for _ in range(STEPS_PER_FRAME):
        if not run.done:
            run.advance()
    for k, (bar, v) in enumerate(zip(bars, run.a)):
        bar.set_height(v)
        bar.set_color(run.color(k))
    if run.range:
        lo, hi = run.range
        bracket.set_data([lo - 0.4, hi + 0.4], [-1.5, -1.5])
    else:
        bracket.set_data([], [])
    if run.pivot:
        pivot_line.set_data([run.range[0] - 0.5, run.range[1] + 0.5], [run.pivot[1]] * 2)
    else:
        pivot_line.set_data([], [])
    state = "sorted" if run.done else "click for a new list"
    status.set_text(f"comparisons {run.comparisons:4d}   swaps {run.swaps:4d}   {state}")
    return (*bars, pivot_line, bracket, status)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.tight_layout(rect=(0, 0.05, 1, 1))
plt.show()
