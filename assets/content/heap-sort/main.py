"""Heap sort: build a max-heap in the array, then move the root behind it n - 1 times.

The array is read as a binary tree: node i has children 2i + 1 and 2i + 2, and in a
max-heap a[parent(i)] >= a[i], so the largest value sits at the root a[0].
  1. Build: sift down every parent, from the last one (n // 2 - 1) back to the root.
  2. Extract: swap the root with the last heap slot (now final), shrink the heap by
     one, and sift the new root down to repair the heap. Repeat.

The sort is a generator that yields one step at a time (compare, swap, extract) while
it reorders the list in place. Top: the array as bars. Bottom: the same array drawn as
the heap tree. Blue in the heap, red comparing, amber swapping, green in final place.
Click for a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection
from matplotlib.patches import Patch

N = 31                                  # 2^5 - 1: a perfect tree with five levels
STEPS_PER_FRAME = 2
BLUE, RED, AMBER, GREEN = "#2346a0", "#d64541", "#a8700f", "#2e9e4f"


def sift_down(a, i, n):
    """Move a[i] down until it is at least as big as its children (heap is a[0..n-1])."""
    while True:
        largest, left, right = i, 2 * i + 1, 2 * i + 2
        if left < n:
            yield ("compare", left, largest)
            if a[left] > a[largest]:
                largest = left
        if right < n:
            yield ("compare", right, largest)
            if a[right] > a[largest]:
                largest = right
        if largest == i:
            return
        a[i], a[largest] = a[largest], a[i]
        yield ("swap", i, largest)
        i = largest


def heap_sort(a):
    """Sort a in place, yielding every step."""
    n = len(a)
    for i in range(n // 2 - 1, -1, -1):    # build the heap bottom-up
        yield from sift_down(a, i, n)
    for end in range(n - 1, 0, -1):
        a[0], a[end] = a[end], a[0]         # the root is the max: park it behind the heap
        yield ("extract", 0, end)
        yield from sift_down(a, 0, end)
    yield ("sorted", 0)


def tree_position(k):
    """Node k of a heap laid out level by level, x in 0..1, y = -level."""
    level = (k + 1).bit_length() - 1
    first = 2 ** level - 1
    return (k - first + 0.5) / 2 ** level, -level


class Run:
    """One shuffled list being sorted, plus what the picture needs to know."""

    def __init__(self):
        self.a = [int(v) for v in np.random.permutation(N) + 1]
        self.steps = heap_sort(self.a)
        self.heap_size = N
        self.hot = {}           # index -> color, for the step just taken
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
            self.hot = dict.fromkeys(idx, RED)
        elif kind in ("swap", "extract"):
            self.swaps += 1
            self.hot = dict.fromkeys(idx, AMBER)
            if kind == "extract":
                self.heap_size = idx[1]      # a[end] is final; the heap shrinks
        else:
            self.heap_size = idx[0]          # a[i..] are all final

    def color(self, k):
        if k in self.hot:
            return self.hot[k]
        return BLUE if k < self.heap_size else GREEN


run = Run()
fig, (ax_bars, ax_tree) = plt.subplots(
    2, 1, figsize=(9, 7), gridspec_kw={"height_ratios": [1, 1.3]})
fig.suptitle("Heap sort: children of i are 2i + 1 and 2i + 2, and a[parent] >= a[child]")
bars = ax_bars.bar(range(N), run.a, width=0.8, color=BLUE)
ax_bars.set_xlim(-1, N)
ax_bars.set_ylim(0, N + 8)
ax_bars.set_xticks([])
ax_bars.set_yticks([])
status = ax_bars.text(0, N + 3, "", family="monospace")

xy = np.array([tree_position(k) for k in range(N)])
edges = LineCollection([[xy[(k - 1) // 2], xy[k]] for k in range(1, N)], zorder=1)
ax_tree.add_collection(edges)
nodes = ax_tree.scatter(xy[:, 0], xy[:, 1], s=330, zorder=2)
values = [ax_tree.text(x, y, "", ha="center", va="center", color="white", fontsize=8,
                       zorder=3) for x, y in xy]
ax_tree.set_xlim(0, 1)
ax_tree.set_ylim(-4.6, 0.6)
ax_tree.axis("off")
legend = [(BLUE, "in the heap"), (RED, "comparing"), (AMBER, "swapping"),
          (GREEN, "in final place")]
fig.legend(handles=[Patch(color=c, label=s) for c, s in legend],
           loc="lower center", ncol=4, frameon=False)


def restart(_event):
    global run
    run = Run()


def update(_frame):
    global run
    if run.done:
        run.rest += 1
        if run.rest > 60:
            run = Run()
    for _ in range(STEPS_PER_FRAME):
        if not run.done:
            run.advance()
    colors = [run.color(k) for k in range(N)]
    for bar, v, c in zip(bars, run.a, colors):
        bar.set_height(v)
        bar.set_color(c)
    nodes.set_facecolors(colors)
    edges.set_colors(["0.35" if k < run.heap_size else "0.85" for k in range(1, N)])
    for text, v in zip(values, run.a):
        text.set_text(str(v))
    phase = ("sorted" if run.done else "building the heap" if run.heap_size == N
             else "extracting the max")
    status.set_text(f"comparisons {run.comparisons:4d}   swaps {run.swaps:4d}   "
                    f"heap size {run.heap_size:2d}   {phase}")
    return (*bars, nodes, edges, *values, status)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.tight_layout(rect=(0, 0.04, 1, 1))
plt.show()
