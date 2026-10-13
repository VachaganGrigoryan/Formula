"""Sorting race: finish time = steps / speed, with steps = comparisons + moves.

Five algorithms sort copies of the same list side by side. Every lane takes the same
number of steps per frame, so the lane needing the fewest operations finishes first.
Each sort is a Python generator that yields ("compare", i, j), ("swap", i, j) or
("write", i); free markers ("sorted", i) and ("pivot", i) only change colors.
Red = comparing, amber = swapping / writing, pink = quick sort's pivot, green = final.
Click the figure to switch the input: random, nearly sorted, reversed, few unique.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

N = 40                # values per list
STEPS_PER_FRAME = 4   # the same for every lane
BLUE, RED, AMBER, GREEN, PINK = "#4a7bd0", "#d64541", "#e8a317", "#2e9e4f", "#e0569b"
DISTRIBUTIONS = ["Random", "Nearly sorted", "Reversed", "Few unique"]


def bubble(a):
    n = len(a)
    for i in range(n):
        swapped = False
        for j in range(n - i - 1):
            yield "compare", j, j + 1
            if a[j] > a[j + 1]:
                a[j], a[j + 1] = a[j + 1], a[j]
                swapped = True
                yield "swap", j, j + 1
        yield "sorted", n - i - 1
        if not swapped:
            return


def insertion(a):
    for i in range(1, len(a)):
        key, j = a[i], i - 1
        while j >= 0:
            yield "compare", j, j + 1   # the key, headed for j + 1, against a[j]
            if a[j] <= key:
                break
            a[j + 1] = a[j]
            yield "write", j + 1
            j -= 1
        a[j + 1] = key
        yield "write", j + 1


def merge(a, lo=0, hi=None):
    hi = len(a) if hi is None else hi
    if hi - lo < 2:
        return
    mid = (lo + hi) // 2
    yield from merge(a, lo, mid)
    yield from merge(a, mid, hi)
    left, right, i, j = a[lo:mid], a[mid:hi], 0, 0
    for k in range(lo, hi):
        if i < len(left) and j < len(right):
            yield "compare", lo + i, mid + j
        if j == len(right) or (i < len(left) and left[i] <= right[j]):
            a[k], i = left[i], i + 1
        else:
            a[k], j = right[j], j + 1
        yield "write", k
        if hi - lo == len(a):
            yield "sorted", k


def quick(a, lo=0, hi=None):
    """Lomuto partition with the last value as pivot, as in the app."""
    hi = len(a) - 1 if hi is None else hi
    if lo >= hi:
        if lo == hi:
            yield "sorted", lo
        return
    yield "pivot", hi
    i = lo
    for j in range(lo, hi):
        yield "compare", j, hi
        if a[j] <= a[hi]:
            if i != j:
                a[i], a[j] = a[j], a[i]
                yield "swap", i, j
            i += 1
    if i != hi:
        a[i], a[hi] = a[hi], a[i]
        yield "swap", i, hi
    yield "pivot", None
    yield "sorted", i               # the pivot is in its final place
    yield from quick(a, lo, i - 1)
    yield from quick(a, i + 1, hi)


def sift_down(a, i, n):
    while True:
        largest = i
        for child in (2 * i + 1, 2 * i + 2):
            if child < n:
                yield "compare", child, largest
                if a[child] > a[largest]:
                    largest = child
        if largest == i:
            return
        a[i], a[largest] = a[largest], a[i]
        yield "swap", i, largest
        i = largest


def heap(a):
    n = len(a)
    for i in range(n // 2 - 1, -1, -1):
        yield from sift_down(a, i, n)
    for end in range(n - 1, 0, -1):
        a[0], a[end] = a[end], a[0]     # the root is the maximum: move it behind the heap
        yield "swap", 0, end
        yield "sorted", end
        yield from sift_down(a, 0, end)


RACERS = [("Bubble", bubble), ("Insertion", insertion), ("Merge", merge),
          ("Quick", quick), ("Heap", heap)]


def make_list(dist, rng):
    """Values 1..N arranged like the app's data menu."""
    a = np.arange(1, N + 1)
    if dist == "Random":
        rng.shuffle(a)
    elif dist == "Nearly sorted":       # sorted, except for three values swapped out of place
        for i, j in rng.integers(N, size=(3, 2)):
            a[[i, j]] = a[[j, i]]
    elif dist == "Reversed":
        a = a[::-1]
    else:
        a = rng.choice([N // 4, N // 2, 3 * N // 4, N], size=N)
    return [int(v) for v in a]


class Lane:
    def __init__(self, name, sort, data):
        self.name, self.a = name, list(data)        # every lane gets its own copy
        self.steps = sort(self.a)
        self.sorted, self.pivot, self.event = [False] * N, None, None
        self.comparisons = self.moves = 0
        self.done = False

    def advance(self):
        """Apply events up to the next comparison or move; markers are free."""
        for event in self.steps:
            if event[0] == "sorted":
                self.sorted[event[1]] = True
            elif event[0] == "pivot":
                self.pivot = event[1]
            else:
                self.event = event
                if event[0] == "compare":
                    self.comparisons += 1
                else:
                    self.moves += 1
                return
        self.done, self.event, self.pivot, self.sorted = True, None, None, [True] * N

    def color(self, i):
        kind, *where = self.event if self.event else ("", None)
        if i in where:
            return RED if kind == "compare" else AMBER
        if i == self.pivot:
            return PINK
        return GREEN if self.sorted[i] else BLUE


rng = np.random.default_rng(4)
race = {"dist": 0, "lanes": [], "hold": 0}
fig, axes = plt.subplots(len(RACERS), 1, figsize=(8, 8))
bar_sets, labels = [], []
for ax in axes:
    bar_sets.append(ax.bar(range(N), np.ones(N), width=0.8, color=BLUE))
    labels.append(ax.text(0.01, 0.95, "", transform=ax.transAxes, va="top", family="monospace",
                          bbox={"fc": "white", "alpha": 0.8, "lw": 0}))
    ax.set_xlim(-1, N)
    ax.set_ylim(0, N * 1.6)
    ax.set_xticks([])
    ax.set_yticks([])


def new_race(_event=None, next_dist=True):
    race["dist"] = (race["dist"] + next_dist) % len(DISTRIBUTIONS)
    data = make_list(DISTRIBUTIONS[race["dist"]], rng)
    race["lanes"] = [Lane(name, sort, data) for name, sort in RACERS]
    race["hold"] = 0
    fig.suptitle(f"Sorting race, {N} values, {DISTRIBUTIONS[race['dist']]} data "
                 f"({STEPS_PER_FRAME} steps per frame; click for the next input)")


def ordinal(k):
    return f"{k}{'st' if k == 1 else 'nd' if k == 2 else 'rd' if k == 3 else 'th'}"


def update(_frame):
    lanes = race["lanes"]
    if all(lane.done for lane in lanes):
        race["hold"] += 1
        if race["hold"] > 90:   # show the result for a moment, then race on a fresh list
            new_race(next_dist=False)
    for lane in lanes:
        for _ in range(STEPS_PER_FRAME):
            if not lane.done:
                lane.advance()
    for lane, bars, label in zip(race["lanes"], bar_sets, labels):
        for i, bar in enumerate(bars):
            bar.set_height(lane.a[i])
            bar.set_color(lane.color(i))
        steps = lane.comparisons + lane.moves
        # Lanes with fewer steps always finish no later, so this ranking is final.
        place = 1 + sum(o.done and o.comparisons + o.moves < steps for o in race["lanes"])
        status = f"{ordinal(place)} place" if lane.done else "running"
        label.set_text(f"{lane.name:<10} comparisons {lane.comparisons:4d}  "
                       f"moves {lane.moves:4d}  steps {steps:4d}  {status}")
    return [b for bars in bar_sets for b in bars] + labels


new_race(next_dist=False)
fig.canvas.mpl_connect("button_press_event", new_race)
anim = FuncAnimation(fig, update, interval=30, blit=False, cache_frame_data=False)
plt.tight_layout()
plt.show()
