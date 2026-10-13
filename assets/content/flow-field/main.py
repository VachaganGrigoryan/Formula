"""Perlin noise flow field: theta(x, y) = N(x * s, y * s, t) * 2 pi * k,
p' = p + v * dt * (cos theta, sin theta).

Improved Perlin noise (Ken Perlin, 2002) turns every point into an angle; 1500
particles follow the angle under them and paint fading trails, colored by heading.
The third noise coordinate is time, so the field slowly drifts. Click for new noise.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.colors import hsv_to_rgb

W, H = 480, 300                  # trail image size in pixels
COUNT = 1500                     # particles
ZOOM, TURNS = 0.008, 1.5         # s and k in the formula
SPEED = 2.0                      # pixels per frame
FADE = 0.03                      # fraction of the trail that fades per frame
EVOLVE = 0.004                   # noise time per frame
BG = np.array([0.04, 0.063, 0.055])


def fade(t):
    """6t^5 - 15t^4 + 10t^3: blends each cell smoothly, so no seams show."""
    return t * t * t * (t * (t * 6 - 15) + 10)


def lerp(t, a, b):
    return a + t * (b - a)


def grad(h, x, y, z):
    """Dot product with one of 12 gradients pointing at the edges of a cube."""
    h = h & 15
    u = np.where(h < 8, x, y)
    v = np.where(h < 4, y, np.where((h == 12) | (h == 14), x, z))
    return np.where(h & 1, -u, u) + np.where(h & 2, -v, v)


def noise(perm, x, y, z):
    """Improved Perlin noise in roughly -1..1; vectorized over x, y, z arrays."""
    x, y, z = np.broadcast_arrays(*(np.asarray(c, dtype=float) for c in (x, y, z)))
    xf, yf, zf = np.floor(x), np.floor(y), np.floor(z)
    xi, yi, zi = (c.astype(int) & 255 for c in (xf, yf, zf))
    x, y, z = x - xf, y - yf, z - zf
    u, v, w = fade(x), fade(y), fade(z)
    p = perm
    a, b = p[xi] + yi, p[xi + 1] + yi                 # hash the 8 cell corners
    aa, ab, ba, bb = p[a] + zi, p[a + 1] + zi, p[b] + zi, p[b + 1] + zi
    return lerp(w,
                lerp(v, lerp(u, grad(p[aa], x, y, z), grad(p[ba], x - 1, y, z)),
                     lerp(u, grad(p[ab], x, y - 1, z), grad(p[bb], x - 1, y - 1, z))),
                lerp(v, lerp(u, grad(p[aa + 1], x, y, z - 1), grad(p[ba + 1], x - 1, y, z - 1)),
                     lerp(u, grad(p[ab + 1], x, y - 1, z - 1),
                          grad(p[bb + 1], x - 1, y - 1, z - 1))))


def angle(n, turns):
    """theta = N * 2 pi * k, with N rescaled from -1..1 to 0..1."""
    return np.clip(n * 0.5 + 0.5, 0, 1) * 2 * np.pi * turns


def permutation(rng):
    return np.tile(rng.permutation(256), 2)        # 512 entries, so p[i + 1] never overflows


rng = np.random.default_rng(1)
state = {"perm": permutation(rng), "z": 0.0}
pos = rng.uniform([0, 0], [W, H], (COUNT, 2))
life = rng.integers(60, 400, COUNT)                 # frames until respawn
trails = np.tile(BG, (H, W, 1))

fig, ax = plt.subplots(figsize=(9, 6))
fig.patch.set_facecolor("#0a100e")
image = ax.imshow(trails, interpolation="bilinear")
ax.axis("off")
ax.set_title(r"$\theta = N(xs,\ ys,\ t) \cdot 2\pi k$     (click for new noise)", color="0.8")


def on_click(_event):
    state["perm"] = permutation(rng)
    trails[:] = BG


fig.canvas.mpl_connect("button_press_event", on_click)


def splat(points, colors, alpha):
    """Blend each point's pixel toward its color; points off the image are skipped."""
    ix, iy = points[:, 0].astype(int), points[:, 1].astype(int)
    ok = (ix >= 0) & (ix < W) & (iy >= 0) & (iy < H)
    ix, iy = ix[ok], iy[ok]
    trails[iy, ix] += (colors[ok] - trails[iy, ix]) * alpha


def update(_frame):
    trails[:] += (BG - trails) * FADE                  # old trails fade toward the background
    theta = angle(noise(state["perm"], pos[:, 0] * ZOOM, pos[:, 1] * ZOOM, state["z"]), TURNS)
    step = SPEED * np.column_stack([np.cos(theta), np.sin(theta)])
    hue = (theta / (2 * np.pi)) % 1.0
    colors = hsv_to_rgb(np.column_stack([hue, np.full(COUNT, 0.65), np.full(COUNT, 0.95)]))
    for f in (0.5, 1.0):                              # two samples per step: no gaps
        splat(pos + f * step, colors, 0.35)
    pos[:] += step
    life[:] -= 1

    # Respawn particles that left the image or ran out of life.
    dead = (life <= 0) | (pos[:, 0] < 0) | (pos[:, 0] >= W) | (pos[:, 1] < 0) | (pos[:, 1] >= H)
    pos[dead] = rng.uniform([0, 0], [W, H], (dead.sum(), 2))
    life[dead] = rng.integers(60, 400, dead.sum())
    state["z"] += EVOLVE
    image.set_data(np.clip(trails, 0, 1))
    return (image,)


anim = FuncAnimation(fig, update, frames=100_000, interval=30, blit=False)
plt.tight_layout()
plt.show()
