"""Euler's formula: e^(i theta) = cos(theta) + i sin(theta).

The Taylor series 1 + i theta + (i theta)^2/2! + ... drawn as arrows placed head to
tail. Each term is the previous one turned a quarter turn and scaled by theta / k, so
the partial sums spiral in to the point at angle theta on the unit circle. At
theta = pi the point is -1: e^(i pi) + 1 = 0.
Use the sliders to change theta and the number of terms.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import Slider


def partial_sums(theta, n):
    """S_0 ... S_n of sum (i theta)^k / k!, each term from the last one."""
    term, total, out = 1 + 0j, 1 + 0j, [1 + 0j]
    for k in range(1, n + 1):
        term *= 1j * theta / k
        total += term
        out.append(total)
    return np.array(out)


fig = plt.figure(figsize=(7, 7.5))
ax = fig.add_axes([0.08, 0.18, 0.84, 0.76])
slider_theta = Slider(fig.add_axes([0.2, 0.09, 0.6, 0.03]), "theta", 0, 2 * np.pi, valinit=2.0)
slider_n = Slider(fig.add_axes([0.2, 0.04, 0.6, 0.03]), "terms", 0, 30, valinit=8, valstep=1)
circle = np.exp(1j * np.linspace(0, 2 * np.pi, 361))


def update(_frame):
    theta, n = slider_theta.val, int(slider_n.val)
    sums = partial_sums(theta, n)
    points = np.concatenate([[0], sums])
    ax.clear()
    ax.set_aspect("equal")
    ax.axhline(0, color="0.85")
    ax.axvline(0, color="0.85")
    ax.plot(circle.real, circle.imag, color="0.6")
    for k in range(len(points) - 1):
        start, end = points[k], points[k + 1]
        color = "#23786e" if k % 2 == 0 else "#a8700f"
        ax.annotate("", xy=(end.real, end.imag), xytext=(start.real, start.imag),
                    arrowprops=dict(arrowstyle="->", color=color, lw=1.8))
    ax.plot([np.cos(theta)], [np.sin(theta)], "o", color="#2346a0", ms=9)
    reach = max(1.2, np.abs(points).max() * 1.1)
    ax.set_xlim(-reach, reach)
    ax.set_ylim(-reach, reach)
    last = sums[-1]
    ax.set_title(f"sum of {n + 1} terms = {last.real:+.4f} {last.imag:+.4f}i,   "
                 f"cos + i sin = {np.cos(theta):+.4f} {np.sin(theta):+.4f}i")
    return []


anim = FuncAnimation(fig, update, interval=50, cache_frame_data=False)
plt.show()
