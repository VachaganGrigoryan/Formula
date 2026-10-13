"""Mandelbrot set: z_{n+1} = z_n^2 + c, z_0 = 0; c is in the set while |z_n| <= 2.

Every pixel is a value of c, colored by its smooth escape count
mu = n + 1 - log2(log2 |z_n|); black points never escape. A test point c travels
around the set and its orbit z_0, z_1, z_2, ... is drawn on top: inside the set it
settles into a cycle, outside it flies off. Left-click zooms in 2x, right-click zooms out.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

BAILOUT = 256.0                      # |z|^2 escape test; large so mu is smooth
W, H = 600, 420                      # image size in pixels
STOPS = np.array([[0.05, 0.07, 0.12], [0.16, 0.45, 0.62], [0.95, 0.80, 0.45],
                  [0.85, 0.33, 0.25], [0.05, 0.07, 0.12]])   # periodic color ramp


def in_main_bulbs(x, y):
    """Main cardioid or period-2 disc: these never escape, so skip iterating them."""
    q = (x - 0.25) ** 2 + y**2
    return (q * (q + (x - 0.25)) <= 0.25 * y**2) | ((x + 1) ** 2 + y**2 <= 0.0625)


def escape(c, max_iter):
    """Smooth escape count for an array of complex c; NaN where z stays bounded."""
    mu = np.full(c.shape, np.nan)
    active = np.flatnonzero(~in_main_bulbs(c.real, c.imag).ravel())
    cs, z = c.ravel()[active], np.zeros(active.size, complex)
    for n in range(max_iter):
        r2 = z.real**2 + z.imag**2
        out = r2 > BAILOUT
        # Continuous escape time: n + 1 - log2(log2 |z|) removes the color bands.
        mu.flat[active[out]] = n + 1 - np.log2(np.log2(r2[out]) / 2)
        active, cs, z = active[~out], cs[~out], z[~out]
        z = z * z + cs
    return mu


def orbit(c, steps=60):
    """The first points of the orbit of c, stopping once it clearly escapes."""
    z, out = 0j, [0j]
    for _ in range(steps):
        z = z * z + c
        out.append(z)
        if abs(z) > 4:
            break
    return np.array(out)


def colorize(mu):
    """Square-root scaled, repeating gradient; inside points are black."""
    t = (np.sqrt(np.maximum(mu - 1, 0)) * 0.06) % 1.0
    pos = np.linspace(0, 1, len(STOPS))
    rgb = np.stack([np.interp(t, pos, STOPS[:, k]) for k in range(3)], axis=-1)
    rgb[np.isnan(mu)] = 0.0
    return rgb


view = {"cx": -0.6, "cy": 0.0, "span": 2.6}      # center and height of the view


def render():
    half_w, half_h = view["span"] * W / H / 2, view["span"] / 2
    x = np.linspace(view["cx"] - half_w, view["cx"] + half_w, W)
    y = np.linspace(view["cy"] + half_h, view["cy"] - half_h, H)
    max_iter = int(300 + 60 * max(0.0, np.log2(2.6 / view["span"])))
    image.set_data(colorize(escape(x[None, :] + 1j * y[:, None], max_iter)))
    image.set_extent((x[0], x[-1], y[-1], y[0]))
    ax.set_xlim(x[0], x[-1])
    ax.set_ylim(y[-1], y[0])
    ax.set_title(f"z -> z^2 + c     zoom x{2.6 / view['span']:.0f}   max_iter {max_iter}")


fig, ax = plt.subplots(figsize=(9, 6.6))
image = ax.imshow(np.zeros((H, W, 3)), origin="upper", interpolation="bilinear")
ax.set_xlabel("Re c")
ax.set_ylabel("Im c")
(path,) = ax.plot([], [], "-o", color="white", lw=1, ms=3, alpha=0.9)
(marker,) = ax.plot([], [], "o", mfc="none", mec="#ffd166", ms=10, mew=2)
label = ax.text(0.02, 0.97, "", transform=ax.transAxes, va="top", family="monospace",
                color="white", bbox=dict(facecolor="black", alpha=0.6))
render()


def on_click(event):
    if event.inaxes is not ax:
        return
    factor = 2.0 if event.button == 1 else 0.5   # left: zoom in, right: zoom out
    view["cx"] = event.xdata + (view["cx"] - event.xdata) / factor
    view["cy"] = event.ydata + (view["cy"] - event.ydata) / factor
    view["span"] /= factor
    render()


fig.canvas.mpl_connect("button_press_event", on_click)


def update(frame):
    # The test point loops around the view center, wandering in and out of the set.
    angle = frame * 0.03
    c = complex(view["cx"], view["cy"]) + view["span"] * (0.18 + 0.1 * np.sin(angle * 2.3)) \
        * np.exp(1j * angle)
    z = orbit(c)
    path.set_data(z.real, z.imag)
    marker.set_data([c.real], [c.imag])
    mu = escape(np.array([c]), 1000)[0]
    fate = "bounded (in the set)" if np.isnan(mu) else f"escapes after {mu:.1f} iterations"
    label.set_text(f"c = {c.real:+.4f} {c.imag:+.4f}i\n{fate}")
    return path, marker, label


anim = FuncAnimation(fig, update, frames=2000, interval=40, blit=False)
plt.tight_layout()
plt.show()
