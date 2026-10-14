//! AVL binary search tree: left subtree < key < right subtree, |h(left) - h(right)| <= 1.
//!
//! Ten keys are inserted with AVL balancing and the tree is drawn with one circle
//! per key. Click to insert a random key and press d to delete the last key you
//! inserted. Every rotation is shown for half a second: the nodes glide from their
//! old place to the new one (a node is identified by its key, so it is tracked
//! through the rotation). When a deletion has to rebalance while it relinks a node
//! with two children, that rebalancing is shown as a single step.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use std::collections::BTreeMap;

use macroquad::prelude::*;

const START_KEYS: [i32; 10] = [50, 30, 70, 20, 40, 60, 80, 35, 45, 65];
const STEP: f32 = 0.5; // seconds each rotation is shown
const WIDTH: f32 = 900.0;
const HEIGHT: f32 = 560.0;
const HEADER: f32 = 30.0;
const RADIUS: f32 = 17.0;

/// key -> (x in 0..1 by in-order rank, depth)
type Layout = BTreeMap<i32, (f32, f32)>;

#[derive(Clone, Copy)]
struct Node {
    left: Option<i32>,
    right: Option<i32>,
    height: i32,
}

/// An AVL tree whose nodes are identified by their (unique) keys.
#[derive(Default)]
struct Tree {
    nodes: BTreeMap<i32, Node>,
    root: Option<i32>,
    /// A layout after every rotation, so the animation can glide between them.
    snaps: Vec<Layout>,
    /// A rotation happened and is not shown yet.
    rotated: bool,
    /// No snapshots while a subtree is detached.
    quiet: bool,
}

impl Tree {
    fn h(&self, k: Option<i32>) -> i32 {
        k.and_then(|k| self.nodes.get(&k)).map_or(0, |n| n.height)
    }

    fn node(&self, k: i32) -> Node {
        self.nodes[&k]
    }

    fn fix(&mut self, k: i32) {
        let n = self.node(k);
        let height = 1 + self.h(n.left).max(self.h(n.right));
        self.nodes.get_mut(&k).unwrap().height = height;
    }

    fn bf(&self, k: i32) -> i32 {
        let n = self.node(k);
        self.h(n.left) - self.h(n.right)
    }

    fn rotate_right(&mut self, k: i32) -> i32 {
        let y = self.node(k).left.unwrap();
        self.nodes.get_mut(&k).unwrap().left = self.node(y).right;
        self.nodes.get_mut(&y).unwrap().right = Some(k);
        self.fix(k);
        self.fix(y);
        y
    }

    fn rotate_left(&mut self, k: i32) -> i32 {
        let y = self.node(k).right.unwrap();
        self.nodes.get_mut(&k).unwrap().right = self.node(y).left;
        self.nodes.get_mut(&y).unwrap().left = Some(k);
        self.fix(k);
        self.fix(y);
        y
    }

    /// Restore |bf| <= 1 at k; returns the new root of that subtree.
    fn balance(&mut self, k: i32) -> i32 {
        self.fix(k);
        let b = self.bf(k);
        if b > 1 {
            let l = self.node(k).left.unwrap();
            if self.bf(l) < 0 {
                let top = self.rotate_left(l);
                self.nodes.get_mut(&k).unwrap().left = Some(top);
                self.snap();
            }
            self.rotated = true;
            return self.rotate_right(k);
        }
        if b < -1 {
            let r = self.node(k).right.unwrap();
            if self.bf(r) > 0 {
                let top = self.rotate_right(r);
                self.nodes.get_mut(&k).unwrap().right = Some(top);
                self.snap();
            }
            self.rotated = true;
            return self.rotate_left(k);
        }
        k
    }

    /// Called right after a subtree was linked back in: show a pending rotation.
    fn flush(&mut self) {
        if self.rotated {
            self.snap();
        }
        self.rotated = false;
    }

    fn insert_at(&mut self, k: Option<i32>, key: i32) -> i32 {
        let Some(k) = k else {
            self.nodes.insert(key, Node { left: None, right: None, height: 1 });
            return key;
        };
        if key < k {
            let sub = self.insert_at(self.node(k).left, key);
            self.nodes.get_mut(&k).unwrap().left = Some(sub);
        } else {
            let sub = self.insert_at(self.node(k).right, key);
            self.nodes.get_mut(&k).unwrap().right = Some(sub);
        }
        self.flush();
        self.balance(k)
    }

    fn remove_min(&mut self, k: i32) -> Option<i32> {
        let Some(lo) = self.node(k).left else {
            return self.node(k).right;
        };
        let sub = self.remove_min(lo);
        self.nodes.get_mut(&k).unwrap().left = sub;
        self.flush();
        Some(self.balance(k))
    }

    fn delete_at(&mut self, k: Option<i32>, key: i32) -> Option<i32> {
        let k = k?;
        if key < k {
            let sub = self.delete_at(self.node(k).left, key);
            self.nodes.get_mut(&k).unwrap().left = sub;
        } else if key > k {
            let sub = self.delete_at(self.node(k).right, key);
            self.nodes.get_mut(&k).unwrap().right = sub;
        } else {
            let Node { left, right, .. } = self.nodes.remove(&k).unwrap();
            let (Some(lo), Some(hi)) = (left, right) else {
                return left.or(right);
            };
            // Two children: the smallest key of the right side takes this place.
            let mut s = hi;
            while let Some(next) = self.node(s).left {
                s = next;
            }
            self.quiet = true;
            let rest = self.remove_min(hi);
            let n = self.nodes.get_mut(&s).unwrap();
            n.left = Some(lo);
            n.right = rest;
            let top = self.balance(s);
            self.quiet = false;
            return Some(top);
        }
        self.flush();
        Some(self.balance(k))
    }

    fn insert(&mut self, key: i32) {
        if self.nodes.contains_key(&key) {
            return;
        }
        self.snaps = vec![self.layout()];
        self.root = Some(self.insert_at(self.root, key));
        self.flush();
        self.finish();
    }

    fn delete(&mut self, key: i32) {
        if !self.nodes.contains_key(&key) {
            return;
        }
        self.snaps = vec![self.layout()];
        self.root = self.delete_at(self.root, key);
        self.flush();
        self.finish();
    }

    fn finish(&mut self) {
        self.rotated = false;
        self.snap();
    }

    /// x by in-order rank, and the depth, of every key.
    fn layout(&self) -> Layout {
        fn walk(t: &Tree, k: Option<i32>, depth: f32, out: &mut Vec<(i32, f32)>) {
            let Some(k) = k else { return };
            let n = t.node(k);
            walk(t, n.left, depth + 1.0, out);
            out.push((k, depth));
            walk(t, n.right, depth + 1.0, out);
        }
        let mut order = Vec::new();
        walk(self, self.root, 0.0, &mut order);
        let n = order.len() as f32;
        order
            .into_iter()
            .enumerate()
            .map(|(i, (k, d))| (k, ((i as f32 + 0.5) / n, d)))
            .collect()
    }

    fn snap(&mut self) {
        if !self.quiet {
            let layout = self.layout();
            self.snaps.push(layout);
        }
    }
}

/// Node positions `clock` seconds into the layouts, interpolated by key.
fn positions(shots: &[Layout], clock: f32) -> Layout {
    let last = shots.len().saturating_sub(1);
    let seg = ((clock / STEP) as usize).min(last.saturating_sub(1));
    let (a, b) = (&shots[seg], &shots[(seg + 1).min(last)]);
    let t = (clock / STEP - seg as f32).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t); // smoothstep
    b.iter()
        .map(|(&key, &(bx, bd))| {
            let (ax, ad) = a.get(&key).copied().unwrap_or((bx, bd));
            (key, (ax + (bx - ax) * t, ad + (bd - ad) * t))
        })
        .collect()
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Binary search tree".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(1);
    let mut tree = Tree::default();
    let mut inserted = Vec::new();
    for key in START_KEYS {
        tree.insert(key);
        inserted.push(key);
    }
    let mut shots = vec![tree.layout()]; // layouts being glided through
    let mut clock = 0.0_f32; // seconds since the first of `shots`

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            let free: Vec<i32> = (-99..=99).filter(|k| !tree.nodes.contains_key(k)).collect();
            if !free.is_empty() {
                let key = free[rand::gen_range(0, free.len())];
                inserted.push(key);
                tree.insert(key);
                shots = std::mem::take(&mut tree.snaps);
                clock = 0.0;
            }
        }
        if is_key_pressed(KeyCode::D) {
            if let Some(key) = inserted.pop() {
                tree.delete(key);
                shots = std::mem::take(&mut tree.snaps);
                clock = 0.0;
            }
        }
        clock += get_frame_time();

        clear_background(Color::from_hex(0x16201c));
        let pos = positions(&shots, clock);
        let levels = tree.h(tree.root).max(1) as f32;
        let at = |x: f32, d: f32| {
            vec2(
                RADIUS + 8.0 + x * (WIDTH - 2.0 * RADIUS - 16.0),
                HEADER + RADIUS + 10.0
                    + d / (levels - 1.0).max(1.0) * (HEIGHT - HEADER - 2.0 * RADIUS - 20.0),
            )
        };
        for (&key, &(x, d)) in &pos {
            let n = tree.node(key);
            for child in [n.left, n.right].into_iter().flatten() {
                if let Some(&(cx, cd)) = pos.get(&child) {
                    let (a, b) = (at(x, d), at(cx, cd));
                    draw_line(a.x, a.y, b.x, b.y, 1.5, Color::from_hex(0x5c6b64));
                }
            }
        }
        for (&key, &(x, d)) in &pos {
            let p = at(x, d);
            draw_circle(p.x, p.y, RADIUS, Color::from_hex(0x243a52));
            draw_circle_lines(p.x, p.y, RADIUS, 1.2, Color::from_hex(0xe6ece8));
            let text = key.to_string();
            let size = measure_text(&text, None, 20, 1.0);
            draw_text(&text, p.x - size.width / 2.0, p.y + size.height / 2.0, 20.0, Color::from_hex(0xe6ece8));
        }
        let header = format!("n = {}, height = {}", tree.nodes.len(), tree.h(tree.root));
        draw_text(&header, 6.0, 21.0, 22.0, WHITE);
        next_frame().await;
    }
}
