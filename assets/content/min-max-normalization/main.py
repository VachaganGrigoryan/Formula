"""Min-max normalization: x_norm = (x - min) / (max - min), height = x_norm * H.

Twelve monthly values are mapped from their raw scale onto 0..1 and then into bar
heights. The August value (the outlier) sweeps up to 500 and back: as the max grows,
every other value is squashed toward 0 and the bars collapse.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection


def min_max(values):
    """Map values onto 0..1. A flat dataset (max == min) maps to all zeros."""
    values = np.asarray(values, dtype=float)
    lo, hi = values.min(), values.max()
    span = hi - lo
    return (values - lo) / span if span > 0 else np.zeros_like(values)


def ping_pong(s):
    """Map any real s onto 0..1..0 at constant speed (a triangle wave)."""
    return 1 - abs(1 - s % 2)


MONTHS = list("JFMAMJJASOND")
OUTLIER = 7          # August is the value that sweeps
RAW_MAX = 500.0      # the raw axis runs from 0 to 500
H_PIXELS = 300       # pretend the chart is 300 px tall
BASE = np.array([55 + 18 * np.sin(0.55 * i) + 10 * np.sin(2.7 * i + 1) for i in range(12)])
LOW, HIGH, MID, GOLD = "#2a9d8f", "#e76f51", "#4a6fa5", "#e9b949"  # min, max, rest, outlier

fig, (ax_axes, ax_bars) = plt.subplots(
    2, 1, figsize=(8, 7), gridspec_kw={"height_ratios": [1, 2]})
fig.suptitle("X_norm = (X - X_min) / (X_max - X_min),   height = X_norm * H")

# Top: the raw axis (y = 1, drawn at v / RAW_MAX) above the normalized axis (y = 0).
ax_axes.set_xlim(-0.04, 1.04)
ax_axes.set_ylim(-0.7, 1.7)
ax_axes.axis("off")
ax_axes.plot([0, 1], [1, 1], color="0.6", lw=1)
ax_axes.plot([0, 1], [0, 0], color="0.6", lw=1)
for tick in range(0, 501, 100):
    ax_axes.text(tick / RAW_MAX, 1.15, str(tick), ha="center", fontsize=8, color="0.4")
ax_axes.text(0, 1.45, "raw values", fontsize=9, color="0.3")
ax_axes.text(0, -0.45, "0", ha="center", color=LOW)
ax_axes.text(1, -0.45, "1", ha="center", color=HIGH)
ax_axes.text(0.5, -0.45, "normalized", ha="center", fontsize=9, color="0.3")
links = LineCollection([], lw=1, alpha=0.5)
ax_axes.add_collection(links)
raw_dots = ax_axes.scatter(np.zeros(12), np.ones(12), s=25, zorder=3)
unit_dots = ax_axes.scatter(np.zeros(12), np.zeros(12), s=25, zorder=3)

# Bottom: the bar chart, with a second axis in pixels.
bars = ax_bars.bar(range(12), np.zeros(12), width=0.7, color=MID)
ax_bars.set_xticks(range(12), MONTHS)
ax_bars.set_ylim(0, 1.05)
ax_bars.set_ylabel("X_norm")
ax_bars.axhline(1, color="0.7", ls="--", lw=1)
ax_bars.secondary_yaxis("right", functions=(lambda u: u * H_PIXELS, lambda p: p / H_PIXELS)) \
    .set_ylabel("pixels")
readout = ax_bars.text(0.01, 0.97, "", transform=ax_bars.transAxes, va="top",
                       family="monospace", fontsize=9)


def update(frame):
    values = BASE.copy()
    values[OUTLIER] = BASE[OUTLIER] + (RAW_MAX - BASE[OUTLIER]) * ping_pong(frame / 90)
    norm = min_max(values)
    lo, hi = values.min(), values.max()
    colors = [LOW if v == lo else HIGH if v == hi else MID for v in values]

    raw_x = values / RAW_MAX
    raw_dots.set_offsets(np.column_stack([raw_x, np.ones(12)]))
    unit_dots.set_offsets(np.column_stack([norm, np.zeros(12)]))
    raw_dots.set_color(colors)
    unit_dots.set_color(colors)
    links.set_segments([[(a, 1), (b, 0)] for a, b in zip(raw_x, norm)])
    links.set_color(colors)

    for i, bar in enumerate(bars):
        bar.set_height(max(norm[i], 0.005))
        bar.set_color(GOLD if i == OUTLIER else MID)
    others = np.delete(norm, OUTLIER)
    readout.set_text(f"Aug = {values[OUTLIER]:5.0f}   min = {lo:.0f}   max = {hi:.0f}\n"
                     f"mean of the other bars: {others.mean():.2f}"
                     f" = {others.mean() * H_PIXELS:.0f} px")
    return links, raw_dots, unit_dots, readout, *bars


anim = FuncAnimation(fig, update, frames=360, interval=30, blit=False)
plt.tight_layout()
plt.show()
