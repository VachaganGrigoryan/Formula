"""Cubic ease-out: f(t) = 1 - (1 - t)^3, with speed f'(t) = 3 (1 - t)^2.

Top left: the curve against the linear diagonal, plus its speed (scaled by 1/3).
Top right: the same progress t drives two dots, linear and eased; the small dots mark
equal slices of time, evenly spaced for linear and bunched up at the end for ease-out.
Bottom: a bar chart growing toward new heights with the eased progress. It loops.
Press E to switch the bars between eased and linear motion.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation


def ease_out_cubic(t):
    """1 - (1 - t)^3 for progress t, clamped to 0..1."""
    u = 1 - np.clip(t, 0, 1)
    return 1 - u * u * u


def ease_out_cubic_speed(t):
    """The derivative 3 (1 - t)^2: 3x linear at the start, 0 at the end."""
    u = 1 - np.clip(t, 0, 1)
    return 3 * u * u


DURATION, HOLD, DT = 1.6, 0.8, 0.04  # seconds
rng = np.random.default_rng(5)
state = {"from": np.zeros(7), "to": rng.uniform(0.15, 1, 7), "elapsed": 0.0, "eased": True}

fig = plt.figure(figsize=(10, 7.5))
ax_curve = fig.add_subplot(2, 2, 1)
ax_race = fig.add_subplot(2, 2, 2)
ax_bars = fig.add_subplot(2, 1, 2)

# 1. The curve, the linear reference and the speed.
u = np.linspace(0, 1, 101)
ax_curve.plot([0, 1], [0, 1], "--", color="0.6", label="linear")
ax_curve.plot(u, ease_out_cubic(u), color="#c07a10", lw=2.5, label="f(t)")
ax_curve.plot(u, ease_out_cubic_speed(u) / 3, color="#d62728", lw=1, alpha=0.7, label="f'(t) / 3")
(stem,) = ax_curve.plot([], [], color="0.5", lw=1)
(linear_dot,) = ax_curve.plot([], [], "o", color="0.5", ms=6)
(eased_dot,) = ax_curve.plot([], [], "o", color="#c07a10", ms=10, mec="white")
ax_curve.set_xlim(0, 1)
ax_curve.set_ylim(0, 1.02)
ax_curve.set_aspect("equal")
ax_curve.set_xlabel("t")
ax_curve.legend(loc="lower right", fontsize=8)

# 2. The race: dots at each tenth of the duration, plus the moving ball.
tenths = np.linspace(0, 1, 11)
for y, f, color in [(1, lambda t: t, "0.5"), (0, ease_out_cubic, "#c07a10")]:
    ax_race.plot([0, 1], [y, y], color="0.85", zorder=0)
    ax_race.plot(f(tenths), np.full(11, y), "o", color=color, alpha=0.4, ms=4)
(ball_linear,) = ax_race.plot([], [], "o", color="0.5", ms=16)
(ball_eased,) = ax_race.plot([], [], "o", color="#c07a10", ms=16)
ax_race.set_xlim(-0.05, 1.05)
ax_race.set_ylim(-0.6, 1.6)
ax_race.set_yticks([0, 1], ["ease-out", "linear"])
ax_race.set_xticks([])
ax_race.set_title("equal slices of time")

# 3. Bars animating from old heights to new targets.
bars = ax_bars.bar(range(7), state["to"], width=0.64, color="#f0b45a")
targets = ax_bars.hlines(state["to"], np.arange(7) - 0.32, np.arange(7) + 0.32, color="0.3")
ax_bars.set_ylim(0, 1.05)
ax_bars.set_xticks([])
label = ax_bars.set_title("t = 0.00   f(t) = 0.00")


def update(_frame):
    state["elapsed"] += DT
    if state["elapsed"] > DURATION + HOLD:  # loop: retarget from where the bars are now
        state["from"], state["to"] = state["to"], rng.uniform(0.15, 1, 7)
        state["elapsed"] = 0.0
    t = min(state["elapsed"] / DURATION, 1.0)
    e = ease_out_cubic(t)

    stem.set_data([t, t], [t, e])
    linear_dot.set_data([t], [t])
    eased_dot.set_data([t], [e])
    ball_linear.set_data([t], [1])
    ball_eased.set_data([e], [0])

    k = e if state["eased"] else t
    for bar, a, b in zip(bars, state["from"], state["to"]):
        bar.set_height(a + (b - a) * k)
        bar.set_color("#f0b45a" if state["eased"] else "0.6")
    targets.set_segments([[(i - 0.32, y), (i + 0.32, y)] for i, y in enumerate(state["to"])])
    label.set_text(f"t = {t:.2f}   f(t) = {e:.2f}   bars: {'eased' if state['eased'] else 'linear'} (E)")
    return stem, linear_dot, eased_dot, ball_linear, ball_eased, targets, label, *bars


def on_key(event):
    if event.key == "e":
        state["eased"] = not state["eased"]


fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=DT * 1000, cache_frame_data=False)
plt.tight_layout()
plt.show()
