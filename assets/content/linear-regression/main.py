"""Line of best fit (least squares):
    m = sum((x - mean_x) * (y - mean_y)) / sum((x - mean_x)^2),   b = mean_y - m * mean_x.

Left: noisy points, the least-squares line (blue) through the mean point, and a trial
line (orange) that swings about the mean point. Its residuals are drawn as segments and
its squared residuals as actual squares. Right: the total squared error of the trial
line as a function of its slope; the bottom of the parabola is the least-squares slope.
Click to add a point, right-click to remove the nearest one.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection, PolyCollection


def fit(x, y):
    """Least-squares slope, intercept and R^2; None if all x are equal."""
    if len(x) < 2:
        return None
    mx, my = x.mean(), y.mean()
    sxx = np.sum((x - mx) ** 2)
    if sxx == 0:  # a vertical line has no slope
        return None
    m = np.sum((x - mx) * (y - my)) / sxx
    b = my - m * mx
    sst = np.sum((y - my) ** 2)
    r2 = 1 - sse(x, y, m, b) / sst if sst > 0 else 1.0
    return m, b, r2


def sse(x, y, m, b):
    """Sum of squared vertical residuals of the line y = m x + b."""
    return np.sum((y - (m * x + b)) ** 2)


rng = np.random.default_rng(9)
xs = rng.uniform(0.6, 9.4, 18)
ys = np.clip(0.6 * xs + 2 + rng.normal(0, 1.2, xs.size), 0.2, 9.8)

fig, (ax, ax_err) = plt.subplots(1, 2, figsize=(12, 5.6), gridspec_kw={"width_ratios": [1, 1]})
ax.set_xlim(0, 10)
ax.set_ylim(0, 10)
ax.set_aspect("equal")
ax.grid(alpha=0.3)
squares = ax.add_collection(PolyCollection([], fc="#ff7f0e18", ec="#ff7f0e66"))
residuals = ax.add_collection(LineCollection([], colors="#ff7f0e", lw=1.5))
(best_line,) = ax.plot([], [], color="#1f77b4", lw=3, label="least squares")
(trial_line,) = ax.plot([], [], color="#ff7f0e", lw=2, ls="--", label="trial line")
dots = ax.scatter([], [], color="black", s=25, zorder=3)
(mean_dot,) = ax.plot([], [], "o", ms=10, mfc="white", mec="#1f77b4", mew=2.5, zorder=4)
ax.legend(loc="upper left")
title = ax.set_title("y = m x + b")

ax_err.set_xlabel("slope of the trial line")
ax_err.set_ylabel("sum of squared residuals")
ax_err.grid(alpha=0.3)
(curve,) = ax_err.plot([], [], color="0.3")
(best_mark,) = ax_err.plot([], [], "o", color="#1f77b4", ms=9)
(trial_mark,) = ax_err.plot([], [], "o", color="#ff7f0e", ms=9)
err_title = ax_err.set_title("sum of squared residuals")


def update(frame):
    result = fit(xs, ys)
    dots.set_offsets(np.c_[xs, ys])
    if result is None:
        title.set_text("add points with different x to fit a line")
        return dots, title
    m, b, r2 = result
    mx, my = xs.mean(), ys.mean()
    grid = np.array([0, 10])
    best_line.set_data(grid, m * grid + b)
    mean_dot.set_data([mx], [my])

    # The trial line pivots on the mean point, so only its slope varies.
    tm = m + 0.7 * np.sin(frame / 25)
    tb = my - tm * mx
    trial_line.set_data(grid, tm * grid + tb)
    on_line = tm * xs + tb
    residuals.set_segments([[(x, y), (x, f)] for x, y, f in zip(xs, ys, on_line)])
    side = on_line - ys  # signed residual; each square has this side length
    squares.set_verts([[(x, y), (x - abs(s), y), (x - abs(s), y + s), (x, y + s)]
                       for x, y, s in zip(xs, ys, side)])
    title.set_text(f"y = {m:.2f}x {'-' if b < 0 else '+'} {abs(b):.2f}    R^2 = {r2:.2f}    n = {len(xs)}")

    slopes = np.linspace(m - 1, m + 1, 101)
    errors = [sse(xs, ys, s, my - s * mx) for s in slopes]
    curve.set_data(slopes, errors)
    ax_err.set_xlim(slopes[0], slopes[-1])
    ax_err.set_ylim(0, max(errors) * 1.05)
    best_mark.set_data([m], [sse(xs, ys, m, b)])
    trial_mark.set_data([tm], [sse(xs, ys, tm, tb)])
    err_title.set_text(f"trial: {sse(xs, ys, tm, tb):.1f}    least squares: {sse(xs, ys, m, b):.1f}")
    return best_line, trial_line, residuals, squares, dots, mean_dot, curve, best_mark, trial_mark


def on_click(event):
    global xs, ys
    if event.inaxes is not ax:
        return
    if event.button == 1:
        xs, ys = np.append(xs, event.xdata), np.append(ys, event.ydata)
    elif event.button == 3 and len(xs) > 0:
        k = int(np.argmin(np.hypot(xs - event.xdata, ys - event.ydata)))
        xs, ys = np.delete(xs, k), np.delete(ys, k)


fig.canvas.mpl_connect("button_press_event", on_click)
anim = FuncAnimation(fig, update, frames=None, interval=40, cache_frame_data=False)
plt.tight_layout()
plt.show()
