"""AVL binary search tree: left subtree < key < right subtree, |h(left) - h(right)| <= 1.

Ten keys are inserted with AVL balancing and the tree is drawn with one circle
per key. Click to insert a random key and press d to delete the last key you
inserted. Every rotation is shown for half a second: the nodes glide from their
old place to the new one (a node is identified by its key, so it is tracked
through the rotation). When a deletion has to rebalance while it relinks a node
with two children, that rebalancing is shown as a single step.

Requirements: pip install matplotlib
Run:          python main.py
"""
import random

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

START_KEYS = [50, 30, 70, 20, 40, 60, 80, 35, 45, 65]
STEP = 0.5        # seconds each rotation is shown
INTERVAL = 30     # milliseconds per frame
RADIUS = 0.032    # node radius in axes units
PAPER = "#16201c"
RULE = "#5c6b64"
NODE = "#243a52"
INK = "#e6ece8"


class Tree:
    """An AVL tree whose nodes are identified by their (unique) keys.

    snaps collects a layout after every rotation, so the animation can glide
    between them."""

    def __init__(self):
        self.left, self.right, self.height = {}, {}, {}
        self.root = None
        self.snaps = []
        self.rotated = False   # a rotation happened and is not shown yet
        self.quiet = False     # no snapshots while a subtree is detached

    def h(self, k):
        return self.height[k] if k is not None else 0

    def fix(self, k):
        self.height[k] = 1 + max(self.h(self.left[k]), self.h(self.right[k]))

    def bf(self, k):
        return self.h(self.left[k]) - self.h(self.right[k])

    def rotate_right(self, k):
        y = self.left[k]
        self.left[k], self.right[y] = self.right[y], k
        self.fix(k)
        self.fix(y)
        return y

    def rotate_left(self, k):
        y = self.right[k]
        self.right[k], self.left[y] = self.left[y], k
        self.fix(k)
        self.fix(y)
        return y

    def balance(self, k):
        """Restore |bf| <= 1 at k; returns the new root of that subtree."""
        self.fix(k)
        b = self.bf(k)
        if b > 1:
            if self.bf(self.left[k]) < 0:
                self.left[k] = self.rotate_left(self.left[k])
                self.snap()
            self.rotated = True
            return self.rotate_right(k)
        if b < -1:
            if self.bf(self.right[k]) > 0:
                self.right[k] = self.rotate_right(self.right[k])
                self.snap()
            self.rotated = True
            return self.rotate_left(k)
        return k

    def flush(self):
        """Called right after a subtree was linked back in: show a pending rotation."""
        if self.rotated:
            self.snap()
        self.rotated = False

    def insert_at(self, k, key):
        if k is None:
            self.left[key], self.right[key], self.height[key] = None, None, 1
            return key
        if key < k:
            self.left[k] = self.insert_at(self.left[k], key)
        else:
            self.right[k] = self.insert_at(self.right[k], key)
        self.flush()
        return self.balance(k)

    def remove_min(self, k):
        if self.left[k] is None:
            return self.right[k]
        self.left[k] = self.remove_min(self.left[k])
        self.flush()
        return self.balance(k)

    def delete_at(self, k, key):
        if k is None:
            return None
        if key < k:
            self.left[k] = self.delete_at(self.left[k], key)
        elif key > k:
            self.right[k] = self.delete_at(self.right[k], key)
        else:
            lo, hi = self.left[k], self.right[k]
            for table in (self.left, self.right, self.height):
                del table[k]
            if lo is None or hi is None:
                return lo if hi is None else hi
            # Two children: the smallest key of the right side takes this place.
            s = hi
            while self.left[s] is not None:
                s = self.left[s]
            self.quiet = True
            rest = self.remove_min(hi)
            self.left[s], self.right[s] = lo, rest
            top = self.balance(s)
            self.quiet = False
            return top
        self.flush()
        return self.balance(k)

    def insert(self, key):
        if key in self.height:
            return
        self.snaps = [self.layout()]
        self.root = self.insert_at(self.root, key)
        self.flush()
        self.finish()

    def delete(self, key):
        if key not in self.height:
            return
        self.snaps = [self.layout()]
        self.root = self.delete_at(self.root, key)
        self.flush()
        self.finish()

    def finish(self):
        self.rotated = False
        self.snap()

    def layout(self):
        """{key: (x in 0..1 by in-order rank, depth)}."""
        order = []

        def walk(k, depth):
            if k is None:
                return
            walk(self.left[k], depth + 1)
            order.append((k, depth))
            walk(self.right[k], depth + 1)

        walk(self.root, 0)
        n = len(order)
        return {k: ((i + 0.5) / n, d) for i, (k, d) in enumerate(order)}

    def snap(self):
        if not self.quiet:
            self.snaps.append(self.layout())

    def depth(self):
        return self.h(self.root)


tree = Tree()
rng = random.Random(1)
inserted = []
for key in START_KEYS:
    tree.insert(key)
    inserted.append(key)

frames = [tree.layout()]   # layouts being glided through
clock = 0.0                # seconds since the first of `frames`


def run(operation, key):
    """Apply an operation and glide through the layouts it produced."""
    global frames, clock
    operation(key)
    frames, clock = tree.snaps, 0.0


def positions():
    """Node positions at the current time, interpolated by key."""
    seg = min(int(clock / STEP), max(len(frames) - 2, 0))
    a = frames[seg]
    b = frames[min(seg + 1, len(frames) - 1)]
    t = min(max(clock / STEP - seg, 0.0), 1.0)
    t = t * t * (3 - 2 * t)  # smoothstep
    out = {}
    for key, (bx, bd) in b.items():
        ax, ad = a.get(key, (bx, bd))
        out[key] = (ax + (bx - ax) * t, ad + (bd - ad) * t)
    return out


fig, ax = plt.subplots(figsize=(9, 5.8))
fig.patch.set_facecolor(PAPER)
ax.set_facecolor(PAPER)
ax.axis("off")
ax.set_xlim(0, 1)
ax.set_ylim(1, 0)
ax.set_aspect("auto")


def update(_frame):
    global clock
    clock += INTERVAL / 1000
    pos = positions()
    levels = max(tree.depth(), 1)
    xy = {k: (0.04 + 0.92 * x, 0.08 + 0.84 * d / max(levels - 1, 1)) for k, (x, d) in pos.items()}
    ax.clear()
    ax.axis("off")
    ax.set_xlim(0, 1)
    ax.set_ylim(1, 0)
    for k, (x, y) in xy.items():
        for child in (tree.left.get(k), tree.right.get(k)):
            if child in xy:
                ax.plot([x, xy[child][0]], [y, xy[child][1]], color=RULE, lw=1.5, zorder=1)
    for k, (x, y) in xy.items():
        ax.scatter([x], [y], s=520, color=NODE, edgecolors=INK, linewidths=1.2, zorder=2)
        ax.text(x, y, str(k), color=INK, ha="center", va="center", fontsize=9, zorder=3)
    ax.set_title(f"n = {len(tree.height)}, height = {tree.depth()}", color="white",
                 family="monospace", fontsize=10)
    return ()


def on_click(event):
    if event.inaxes is not ax:
        return
    free = [k for k in range(-99, 100) if k not in tree.height]
    if free:
        key = rng.choice(free)
        inserted.append(key)
        run(tree.insert, key)


def on_key(event):
    if event.key == "d" and inserted:
        run(tree.delete, inserted.pop())


fig.canvas.mpl_connect("button_press_event", on_click)
fig.canvas.mpl_connect("key_press_event", on_key)
anim = FuncAnimation(fig, update, frames=None, interval=INTERVAL, cache_frame_data=False)
plt.show()
