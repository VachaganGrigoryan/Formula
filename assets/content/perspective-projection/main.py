"""Perspective projection: x' = f * x / z + cx,  y' = f * y / z + cy.

A cube, a floor grid and a cloud of random 3D points are rotated, pushed in front
of the camera, and flattened onto the screen by dividing by depth z: far points
shrink toward the center. The inset shows the highlighted point's ray from above.
Drag to orbit, scroll to move the camera, Up/Down change f, D toggles a dolly zoom.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.collections import LineCollection

NEAR = 0.05  # points closer than this to the eye are not drawn


def rotate(p, yaw, pitch):
    """Rotate (n, 3) points around the vertical axis (yaw), then the horizontal (pitch)."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    x, z = x * np.cos(yaw) + z * np.sin(yaw), -x * np.sin(yaw) + z * np.cos(yaw)
    y, z = y * np.cos(pitch) - z * np.sin(pitch), y * np.sin(pitch) + z * np.cos(pitch)
    return np.stack([x, y, z], axis=-1)


def project(p, f):
    """Divide by depth: (x, y, z) -> (f x / z, f y / z). Points behind NEAR become nan."""
    z = np.where(p[..., 2] > NEAR, p[..., 2], np.nan)
    return np.stack([f * p[..., 0] / z, f * p[..., 1] / z], axis=-1)


# The scene: cube edges, a floor grid at y = -1, and 70 random points in the cube.
corners = np.array([[x, y, z] for z in (-1, 1) for y in (-1, 1) for x in (-1, 1)], float)
edges = [(i, j) for i in range(8) for j in range(i + 1, 8) if bin(i ^ j).count("1") == 1]
cube = np.array([[corners[i], corners[j]] for i, j in edges])
u = np.linspace(-1.6, 1.6, 9)
floor = np.array([[[a, -1, -1.6], [a, -1, 1.6]] for a in u] +
                 [[[-1.6, -1, a], [1.6, -1, a]] for a in u])
scatter = np.random.default_rng(3).uniform(-1, 1, (70, 3))

cam = {"f": 1.0, "d": 4.0, "yaw": 0.6, "pitch": -0.35, "spin": True, "drag": None}

fig = plt.figure(figsize=(9, 6))
ax = fig.add_axes([0, 0, 1, 1])
ax.set_xlim(-0.75, 0.75)
ax.set_ylim(-0.5, 0.5)
ax.set_aspect("equal")
ax.axis("off")
floor_lines = ax.add_collection(LineCollection([], colors="0.85", lw=1))
cube_lines = ax.add_collection(LineCollection([], colors="0.45", lw=1.2))
dots = ax.scatter([], [], edgecolors="white", linewidths=0.8)
(hero,) = ax.plot([], [], "o", color="#d0453a", ms=10, mec="white", mew=1.5, zorder=5)
info = ax.text(-0.73, 0.48, "", va="top", family="monospace")

# Inset: the highlighted point's ray, seen from above (the x-z plane).
top = fig.add_axes([0.68, 0.04, 0.3, 0.3])
top.set_title("seen from above", fontsize=9, loc="left")
top.set_xticks([])
top.set_yticks([])
(plane,) = top.plot([], [], color="#2346a0", lw=2)
(ray,) = top.plot([], [], "--", color="#d0453a", lw=1)
(pt,) = top.plot([], [], "o", color="#d0453a")
(hit,) = top.plot([], [], "o", color="black", ms=4)
top.plot([0], [0], "o", color="black", ms=5)
top.text(0, -0.5, "eye", ha="center", va="top", fontsize=8)
hit_label = top.text(0, 0, "x'", fontsize=9, ha="right", va="bottom")


def depth_colors(z, near, far):
    """Blend from dark blue (near) to light grey (far)."""
    t = np.clip((z - near) / (far - near), 0, 1)[:, None]
    return (1 - t) * np.array([0.14, 0.27, 0.63, 1]) + t * np.array([0.8, 0.8, 0.8, 1])


def update(_frame):
    if cam["spin"]:
        cam["yaw"] += 0.02
    f, d = cam["f"], cam["d"]

    def to_camera(p):
        return rotate(p, cam["yaw"], cam["pitch"]) + np.array([0, 0, d])

    floor_lines.set_segments(project(to_camera(floor), f))
    cube_lines.set_segments(project(to_camera(cube), f))

    # Painter's algorithm: far points first, so near ones cover them.
    pts = to_camera(scatter)
    order = np.argsort(-pts[:, 2])
    screen = project(pts[order], f)
    dots.set_offsets(screen)
    dots.set_sizes((np.clip(0.045 * f * 400 / pts[order, 2], 1.5, 18) * 2) ** 2)
    dots.set_facecolors(depth_colors(pts[order, 2], d - 1.8, d + 1.8))
    hero.set_data(*project(pts[0], f)[:, None])
    info.set_text(f"f = {f:.2f}   camera distance = {d:.1f}\n"
                  "drag: orbit   scroll: distance   Up/Down: f   D: dolly zoom")

    # The ray from the eye to (x, z) crosses the image plane at x' = f x / z.
    x, z = pts[0, 0], pts[0, 2]
    max_z = d + 2
    top.set_xlim(-0.3, max_z)
    top.set_ylim(-2.2, 2.2)
    plane.set_data([f, f], [-2.2, 2.2])
    ray.set_data([0, z], [0, x])
    pt.set_data([z], [x])
    hit.set_data([f], [f * x / z])
    hit_label.set_position((f - 0.1, f * x / z + 0.1))
    return floor_lines, cube_lines, dots, hero, info, plane, ray, pt, hit, hit_label


def on_press(event):
    if event.inaxes is ax:
        cam["drag"] = (event.x, event.y)


def on_move(event):
    if cam["drag"] is not None:
        cam["yaw"] += (event.x - cam["drag"][0]) * 0.01
        cam["pitch"] = np.clip(cam["pitch"] - (event.y - cam["drag"][1]) * 0.01, -1.4, 1.4)
        cam["drag"], cam["spin"] = (event.x, event.y), False


def on_release(_event):
    cam["drag"] = None


def on_scroll(event):
    cam["d"] = np.clip(cam["d"] - 0.5 * event.step, 2.2, 12)


def on_key(event):
    if event.key == "up":
        cam["f"] = min(cam["f"] + 0.1, 2.5)
    elif event.key == "down":
        cam["f"] = max(cam["f"] - 0.1, 0.3)
    elif event.key == "d":
        # Dolly zoom: move the camera and scale f with it, so the cube keeps its size.
        cam["d"] = 10.0 if cam["d"] < 7 else 3.0
        cam["f"] = 0.25 * cam["d"]


for name, handler in [("button_press_event", on_press), ("motion_notify_event", on_move),
                      ("button_release_event", on_release), ("scroll_event", on_scroll),
                      ("key_press_event", on_key)]:
    fig.canvas.mpl_connect(name, handler)

anim = FuncAnimation(fig, update, interval=30, cache_frame_data=False)
plt.show()
