"""Relative luminance (WCAG): Y = 0.2126 R + 0.7152 G + 0.0722 B on linearized sRGB,
and contrast = (Y_light + 0.05) / (Y_dark + 0.05).

Top: a heatmap where every cell shows its Y, written in black or white, whichever
contrasts more. Bottom: the selected cell's swatch with both text colors and their
contrast ratios, and a stacked bar of the red, green and blue contributions to Y with
the switch point (Y = 0.179) marked. The selection walks through the cells until you
click one. Keys: L toggles linearization, N toggles the naive "always white" labels.

Requirements: pip install numpy matplotlib
Run:          python main.py
"""
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.colors import hsv_to_rgb
from matplotlib.patches import Rectangle

WEIGHTS = np.array([0.2126, 0.7152, 0.0722])  # the eye is most sensitive to green
SWITCH = np.sqrt(1.05 * 0.05) - 0.05          # black and white text contrast equally here


def srgb_to_linear(c):
    """Undo the sRGB transfer curve: encoded 0..1 value -> linear light."""
    c = np.asarray(c, dtype=float)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def contributions(rgb8, linearize=True):
    """The three weighted terms of Y for an 8-bit color."""
    c = np.asarray(rgb8, dtype=float) / 255
    return WEIGHTS * (srgb_to_linear(c) if linearize else c)


def contrast(a, b):
    """WCAG contrast ratio between two luminances, 1:1 up to 21:1."""
    return (max(a, b) + 0.05) / (min(a, b) + 0.05)


def prefers_black_text(y):
    return contrast(y, 0.0) >= contrast(y, 1.0)


COLS, ROWS = 12, [(0.3, 1.0), (0.65, 1.0), (1.0, 0.95), (1.0, 0.7), (0.9, 0.42)]  # (sat, value)
COLORS = [[np.round(hsv_to_rgb([c / COLS, s, v]) * 255).astype(int) for c in range(COLS)] for s, v in ROWS]
state = {"linearize": True, "naive": False, "picked": None}
plt.rcParams["keymap.yscale"] = []  # free the L key (it toggles a log y-axis by default)

fig, (ax_map, ax_panel) = plt.subplots(2, 1, figsize=(10, 7), gridspec_kw={"height_ratios": [5, 3]})
ax_map.set_xlim(0, COLS)
ax_map.set_ylim(len(ROWS), 0)
ax_map.axis("off")
labels = []
for r, row in enumerate(COLORS):
    for c, rgb in enumerate(row):
        ax_map.add_patch(Rectangle((c + 0.04, r + 0.04), 0.92, 0.92, color=rgb / 255))
        labels.append(ax_map.text(c + 0.5, r + 0.5, "", ha="center", va="center", fontsize=10))
outline = ax_map.add_patch(Rectangle((0, 0), 1, 1, fill=False, ec="black", lw=3))

ax_panel.set_xlim(0, 1)
ax_panel.set_ylim(0, 1)
ax_panel.axis("off")
swatch = ax_panel.add_patch(Rectangle((0, 0.05), 0.32, 0.9))
black_text = ax_panel.text(0.03, 0.68, "", color="black", fontsize=13)
white_text = ax_panel.text(0.03, 0.32, "", color="white", fontsize=13)
ax_panel.add_patch(Rectangle((0.38, 0.62), 0.6, 0.18, color="0.9"))
segments = [ax_panel.add_patch(Rectangle((0.38, 0.62), 0, 0.18, color=c))
            for c in ("#d64541", "#2ea043", "#3868d6")]
ax_panel.plot([0.38 + 0.6 * SWITCH] * 2, [0.57, 0.85], color="black", lw=2)
ax_panel.text(0.38 + 0.6 * SWITCH, 0.88, f"text flips at Y = {SWITCH:.3f}", ha="center", fontsize=9)
(y_mark,) = ax_panel.plot([], [], color="#c07a10", lw=3)
readout = ax_panel.text(0.38, 0.4, "", family="monospace", fontsize=10, va="top")


def update(frame):
    picked = state["picked"] or divmod(frame // 12 % (COLS * len(ROWS)), COLS)
    row, col = picked
    for k, label in enumerate(labels):
        rgb = COLORS[k // COLS][k % COLS]
        y = contributions(rgb, state["linearize"]).sum()
        label.set_text(f"{y:.2f}")
        label.set_color("white" if state["naive"] or not prefers_black_text(y) else "black")
    outline.set_xy((col, row))

    rgb = COLORS[row][col]
    parts = contributions(rgb, state["linearize"])
    y = parts.sum()
    swatch.set_color(rgb / 255)
    best = "black" if prefers_black_text(y) else "white"
    black_text.set_text(f"Black text  {contrast(y, 0):.1f}:1" + ("  (best)" if best == "black" else ""))
    white_text.set_text(f"White text  {contrast(y, 1):.1f}:1" + ("  (best)" if best == "white" else ""))
    start = 0.38
    for seg, part in zip(segments, parts):
        seg.set_x(start)
        seg.set_width(0.6 * part)
        start += 0.6 * part
    y_mark.set_data([0.38 + 0.6 * y] * 2, [0.55, 0.8])
    lin = srgb_to_linear(rgb / 255) if state["linearize"] else rgb / 255
    readout.set_text(f"RGB {rgb[0]} {rgb[1]} {rgb[2]}, channels "
                     f"{'linearized (L)' if state['linearize'] else 'as is, /255 (L)'}\n"
                     f"Y = 0.2126*{lin[0]:.2f} + 0.7152*{lin[1]:.2f} + 0.0722*{lin[2]:.2f} = {y:.3f}\n"
                     f"labels: {'always white (N)' if state['naive'] else 'best contrast (N)'}")
    return [*labels, outline, swatch, black_text, white_text, *segments, y_mark, readout]


def on_click(event):
    if event.inaxes is ax_map:
        state["picked"] = (int(min(event.ydata, len(ROWS) - 0.01)), int(min(event.xdata, COLS - 0.01)))


def on_key(event):
    if event.key in ("l", "n"):
        name = "linearize" if event.key == "l" else "naive"
        state[name] = not state[name]


fig.canvas.mpl_connect("button_press_event", on_click)
fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=40, cache_frame_data=False)
plt.tight_layout()
plt.show()
