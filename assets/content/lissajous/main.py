"""Lissajous figures: x(t) = A sin(a t + delta pi),  y(t) = B sin(b t).

One sine wave drives the horizontal position and another the vertical one. The dots
on the top and left tracks show each wave alone; the point they meet at traces the
figure. delta drifts slowly forward, so the knot morphs and seems to turn in 3D.
Use the sliders to change the frequencies a and b and the drift rate; Space pauses.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import Slider

TAU = 2 * np.pi
A, B = 1.0, 1.0      # amplitudes
TRACK = 1.22         # where the two single-wave tracks sit


def lissajous(t, a, b, delta, amp_x=A, amp_y=B):
    """The point at parameter t; delta is the phase of x in units of pi."""
    return amp_x * np.sin(a * t + delta * np.pi), amp_y * np.sin(b * t)


state = {"t": 0.0, "delta": 0.5, "playing": True}

fig = plt.figure(figsize=(7, 8.2))
ax = fig.add_axes([0.05, 0.18, 0.9, 0.74])
ax.set_xlim(-1.4, 1.15)
ax.set_ylim(-1.3, 1.4)
ax.set_aspect("equal")
ax.axis("off")
ax.add_patch(plt.Rectangle((-1.08, -1.08), 2.16, 2.16, fill=False, ec="0.8"))
ax.plot([-1, 1], [TRACK, TRACK], color="#d0453a", alpha=0.3)
ax.plot([-TRACK, -TRACK], [-1, 1], color="#2f7d4f", alpha=0.3)

(faint,) = ax.plot([], [], ":", color="0.6", lw=1)
(trail,) = ax.plot([], [], color="#2346a0", lw=2.2)
(x_link,) = ax.plot([], [], "--", color="#d0453a", lw=1)
(y_link,) = ax.plot([], [], "--", color="#2f7d4f", lw=1)
(x_dot,) = ax.plot([], [], "o", color="#d0453a")
(y_dot,) = ax.plot([], [], "o", color="#2f7d4f")
(tip,) = ax.plot([], [], "o", color="black", ms=5)
ratio = ax.text(1.08, -1.28, "", ha="right", fontsize=22, color="#2346a0")
info = fig.text(0.05, 0.975, "", va="top", family="monospace")

slider_a = Slider(fig.add_axes([0.2, 0.12, 0.6, 0.025]), "a", 1, 9, valinit=3, valstep=1)
slider_b = Slider(fig.add_axes([0.2, 0.08, 0.6, 0.025]), "b", 1, 9, valinit=2, valstep=1)
slider_drift = Slider(fig.add_axes([0.2, 0.04, 0.6, 0.025]), "drift (pi/s)", 0, 0.3,
                      valinit=0.05)


def on_key(event):
    if event.key == " ":
        state["playing"] = not state["playing"]


fig.canvas.mpl_connect("key_press_event", on_key)


def update(_frame):
    dt = 0.03
    if state["playing"]:
        state["t"] = (state["t"] + 1.0 * dt) % TAU          # one period every ~6 s
    state["delta"] = (state["delta"] + slider_drift.val * dt) % 2
    a, b, t, delta = int(slider_a.val), int(slider_b.val), state["t"], state["delta"]

    full = np.linspace(0, TAU, 721)
    faint.set_data(*lissajous(full, a, b, delta))
    trail.set_data(*lissajous(full[full <= t], a, b, delta))
    x, y = lissajous(t, a, b, delta)
    x_dot.set_data([x], [TRACK])
    y_dot.set_data([-TRACK], [y])
    x_link.set_data([x, x], [TRACK, y])
    y_link.set_data([-TRACK, x], [y, y])
    tip.set_data([x], [y])
    ratio.set_text(f"{a}:{b}")
    info.set_text(f"x(t) = A sin({a}t + {delta:.2f} pi)    y(t) = B sin({b}t)")
    return faint, trail, x_link, y_link, x_dot, y_dot, tip, ratio, info


anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.show()
