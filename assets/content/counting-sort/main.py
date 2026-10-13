"""Counting sort: sort small integers without comparing them, in O(n + k) time.

  1. Tally:   count[v] += 1 for every value v in the input.
  2. Prefix:  count[v] += count[v - 1], so count[v] is where v's block ends in the output.
  3. Place:   walking the input right to left, count[v] -= 1, output[count[v]] = v.
              (Right to left keeps equal values in their original order: a stable sort.)
  4. Copy the output back into the input.

The sort is a generator that yields one step at a time while it fills count[] and
output[]. Top: the input. Middle: the tally histogram count[v], with its numbers.
Bottom: the output array filling up. Red marks the value being read, amber the tally
or slot being written, green values in their final place. Click for a new list.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Patch

N, MAX_VALUE = 20, 6
BLUE, RED, AMBER, GREEN = "#2346a0", "#d64541", "#a8700f", "#2e9e4f"
PHASES = {"count": "1. tally: count[a[i]] += 1",
          "prefix": "2. running sum: count[v] += count[v - 1]",
          "place": "3. place, right to left: count[v] -= 1; output[count[v]] = v",
          "write": "4. copy the output back into a"}


def counting_sort(a, count, output):
    """Sort a using count (max(a) + 1 zeros) and output (len(a) slots); yield every step."""
    for i, v in enumerate(a):              # 1. tally
        count[v] += 1
        yield ("count", i)
    for v in range(1, len(count)):         # 2. count[v] = end of v's block
        count[v] += count[v - 1]
        yield ("prefix", v)
    for i in reversed(range(len(a))):      # 3. place, backwards for stability
        v = a[i]
        count[v] -= 1
        output[count[v]] = v
        yield ("place", i, count[v])
    for i, v in enumerate(output):         # 4. copy back
        a[i] = v
        yield ("write", i)


class Run:
    """One random list being sorted, plus what the picture needs to know."""

    def __init__(self):
        self.a = [int(v) for v in np.random.randint(1, MAX_VALUE + 1, N)]
        self.count = [0] * (max(self.a) + 1)
        self.output = [0] * N               # 0 = empty slot (values start at 1)
        self.steps = counting_sort(self.a, self.count, self.output)
        self.sorted = [False] * N
        self.hot_in, self.hot_count, self.hot_out = {}, {}, {}
        self.phase, self.writes, self.done, self.rest = "count", 0, False, 0

    def advance(self):
        self.hot_in, self.hot_count, self.hot_out = {}, {}, {}
        step = next(self.steps, None)
        if step is None:
            self.done = True
            return
        kind, i, *rest = step
        self.phase = kind
        if kind == "count":
            self.hot_in = {i: RED}
            self.hot_count = {self.a[i]: AMBER}
        elif kind == "prefix":
            self.hot_count = {i - 1: RED, i: AMBER}
        elif kind == "place":
            self.writes += 1
            self.hot_in = {i: RED}
            self.hot_count = {self.a[i]: AMBER}
            self.hot_out = {rest[0]: AMBER}
        else:
            self.writes += 1
            self.sorted[i] = True
            self.hot_in = {i: AMBER}


run = Run()
fig, (ax_in, ax_count, ax_out) = plt.subplots(
    3, 1, figsize=(9, 7.5), gridspec_kw={"height_ratios": [1.2, 1, 0.9]})
in_bars = ax_in.bar(range(N), run.a, width=0.8)
ax_in.set_ylim(0, MAX_VALUE + 2)
status = ax_in.text(-0.5, MAX_VALUE + 0.9, "", family="monospace")
ax_in.set_title("input a", loc="left", fontsize=10)
count_bars = ax_count.bar(range(MAX_VALUE + 1), [0] * (MAX_VALUE + 1), width=0.6)
count_labels = [ax_count.text(v, 0, "", ha="center", va="bottom")
                for v in range(MAX_VALUE + 1)]
ax_count.set_xticks(range(MAX_VALUE + 1))
ax_count.set_title("count[v]", loc="left", fontsize=10)
out_bars = ax_out.bar(range(N), [0] * N, width=0.8, color=AMBER)
ax_out.set_ylim(0, MAX_VALUE + 0.5)
ax_out.set_title("output", loc="left", fontsize=10)
for ax in (ax_in, ax_out):
    ax.set_xlim(-1, N)
    ax.set_xticks([])
ax_count.set_yticks([])
legend = [(BLUE, "unsorted"), (RED, "reading a value"), (AMBER, "counting / placing"),
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
        if run.rest > 25:
            run = Run()
    if not run.done:
        run.advance()
    for i, (bar, v) in enumerate(zip(in_bars, run.a)):
        bar.set_height(v)
        bar.set_color(run.hot_in.get(i, GREEN if run.sorted[i] else BLUE))
    k = len(run.count)
    peak = max(max(run.count), 1)
    ax_count.set_ylim(0, peak * 1.3)
    for v, (bar, label) in enumerate(zip(count_bars, count_labels)):
        c = run.count[v] if v < k else 0
        bar.set_height(c)
        bar.set_color(run.hot_count.get(v, "#7f93c9"))
        label.set_position((v, c))
        label.set_text(str(c) if v < k else "")
    for i, (bar, v) in enumerate(zip(out_bars, run.output)):
        bar.set_height(v)
        bar.set_color(run.hot_out.get(i, BLUE))
    state = "sorted" if run.done else PHASES[run.phase]
    status.set_text(f"comparisons 0   writes {run.writes:2d}   {state}")
    return (*in_bars, *count_bars, *count_labels, *out_bars, status)


fig.canvas.mpl_connect("button_press_event", restart)
anim = FuncAnimation(fig, update, interval=100, cache_frame_data=False)
plt.tight_layout(rect=(0, 0.04, 1, 1))
plt.show()
