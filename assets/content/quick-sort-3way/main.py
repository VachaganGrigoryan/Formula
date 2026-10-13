"""Quick sort (3-way): Dijkstra's Dutch-national-flag partition.

Pivot v = a[lo]. Three pointers lt <= i <= gt grow four regions:
    a[lo..lt-1] < v,  a[lt..i-1] == v,  a[i..gt] not seen yet,  a[gt+1..hi] > v.
Look at a[i]: smaller -> swap it to lt; larger -> swap it to gt; equal -> leave it.
When i passes gt, every copy of v is in its final place at once; recurse on the
< and > parts only. On lists with few distinct values that saves most of the work.

The sort is a generator that yields one step at a time while it reorders the list in
place. Blue unsorted, red comparing, amber swapping, pink pivot, green in final place;
the strip under the bars shows the < / = / > regions. Click for a new list (it
alternates between few unique values and all distinct values).

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


def quick_sort_3way(a, lo, hi):
    """Sort a[lo..hi] in place with a three-way partition, yielding every step."""
    if lo >= hi:
        if lo == hi:
            yield ("sorted", lo, lo)
        return
    v = a[lo]
    lt, i, gt = lo, lo + 1, hi
    while i <= gt:
        yield ("compare", lo, lt, i, gt, hi)    # one three-way comparison of a[i] with v
        if a[i] < v:
            a[lt], a[i] = a[i], a[lt]
            yield ("swap", lt, i)
            lt, i = lt + 1, i + 1
        elif a[i] > v:
            if i != gt:
                a[i], a[gt] = a[gt], a[i]
                yield ("swap", i, gt)
            gt -= 1
        else:
            i += 1
    yield ("sorted", lt, gt)                    # the whole band of pivot copies is final
    yield from quick_sort_3way(a, lo, lt - 1)
    yield from quick_sort_3way(a, gt + 1, hi)


def make_list(few_unique):
    if few_unique:                              # four levels, like the app's "Few unique"
        return [int(v) for v in np.random.choice([N // 4, N // 2, 3 * N // 4, N], N)]
    return [int(v) for v in np.random.permutation(N) + 1]


class Run:
    """One list being sorted, plus what the picture needs to know."""

    def __init__(self, few_unique):
        self.few_unique = few_unique
        self.a = make_list(few_unique)
        self.steps = quick_sort_3way(self.a, 0, N - 1)
        self.sorted = [False] * N
        self.hot = {}           # index -> color, for the step just taken
        self.flag = None        # (lo, lt, i, gt, hi) of the partition in progress
        self.pivot = None       # (index, value)
        self.comparisons = self.swaps = 0
        self.done = False
        self.rest = 0

    def advance(self):
        self.hot = {}
        step = next(self.steps, None)
        if step is None:
            self.done = True
            return
        kind, *idx = step
        if kind == "compare":
            self.comparisons += 1
            self.flag = tuple(idx)
            lt, i = idx[1], idx[2]
            self.pivot = (lt, self.a[lt])
            self.hot = {i: RED, lt: RED}
        elif kind == "swap":
            self.swaps += 1
            self.hot = dict.fromkeys(idx, AMBER)
        else:
            for k in range(idx[0], idx[1] + 1):
                self.sorted[k] = True
            self.flag = self.pivot = None

    def color(self, k):
        if k in self.hot:
            return self.hot[k]
        if self.pivot and k == self.pivot[0]:
            return PINK
        if self.sorted[k]:
            return GREEN
        if self.flag and not self.flag[0] <= k <= self.flag[4]:
            return FADED
        return BLUE

    def region(self, k):
        """Which band of the flag slot k is in: '<', '=', '?', '>' or '' outside."""
        if not self.flag:
            return ""
        lo, lt, i, gt, hi = self.flag
        if not lo <= k <= hi:
            return ""
        return "<" if k < lt else "=" if k < i else "?" if k <= gt else ">"


run = Run(few_unique=True)
fig, ax = plt.subplots(figsize=(9, 5.5))
fig.suptitle("Quick sort (3-way): a[lo..lt-1] < v,  a[lt..gt] = v,  a[gt+1..hi] > v")
bars = ax.bar(range(N), run.a, width=0.8, color=BLUE)
strip = ax.bar(range(N), [1.6] * N, bottom=-2.4, width=1.0, color="none")
labels = [ax.text(k, -1.6, "", ha="center", va="center", fontsize=8) for k in range(N)]
ax.set_xlim(-1, N)
ax.set_ylim(-3, N + 6)
ax.set_xticks([])
ax.set_yticks([])
(pivot_line,) = ax.plot([], [], "--", color=PINK, lw=1.2)
status = ax.text(0, N + 3, "", family="monospace")
STRIP = {"<": "0.78", "=": to_rgba(PINK, 0.45), "?": "none", ">": "0.55", "": "none"}
legend = [(BLUE, "unsorted"), (RED, "comparing"), (AMBER, "swapping"),
          (PINK, "pivot"), (GREEN, "in final place")]
fig.legend(handles=[Patch(color=c, label=s) for c, s in legend],
           loc="lower center", ncol=5, frameon=False)


def restart(_event):
    global run
    run = Run(not run.few_unique)


def update(_frame):
    global run
    if run.done:
        run.rest += 1
        if run.rest > 60:
            run = Run(run.few_unique)
    for _ in range(STEPS_PER_FRAME):
        if not run.done:
            run.advance()
    for k, (bar, v) in enumerate(zip(bars, run.a)):
        bar.set_height(v)
        bar.set_color(run.color(k))
        band = run.region(k)
        strip[k].set_color(STRIP[band])
        labels[k].set_text("" if band == "?" else band)
    if run.pivot:
        lo, hi = run.flag[0], run.flag[4]
        pivot_line.set_data([lo - 0.5, hi + 0.5], [run.pivot[1]] * 2)
    else:
        pivot_line.set_data([], [])
    data = "few unique values" if run.few_unique else "all distinct"
    state = "sorted" if run.done else "click: other data"
    status.set_text(f"comparisons {run.comparisons:4d}   swaps {run.swaps:4d}   "
                    f"{data}   {state}")
    return (*bars, *strip, *labels, pivot_line, status)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.tight_layout(rect=(0, 0.05, 1, 1))
plt.show()
