//! The binary search tree viewport: every insert, delete, find and traversal
//! is recorded as snapshot frames and replayed, with rotations tweening each
//! node to its new position by id.

use egui::{Align2, FontId, Rect, Stroke, pos2, vec2};

use super::tree::{Frame, Rotation, Tree};
use crate::formula::{Formula, MathText, Palette};
use crate::formulas::replay::{self, Player, Replay};
use crate::rng::Rng;

/// Frames per second at the start.
const SPEED: f32 = 2.5;
/// Height of the readout strip above the tree.
const HEADER: f32 = 30.0;
/// Height of the caption strip below the tree.
const FOOTER: f32 = 28.0;
/// Largest node radius.
const RADIUS: f32 = 15.0;
/// The keys the tool starts with.
const START_KEYS: [i32; 10] = [50, 30, 70, 20, 40, 60, 80, 35, 45, 65];

/// The index of the frame being shown; one op advances to the next frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameIndex(pub usize);

impl Replay for FrameIndex {
    type Op = ();
    fn apply(&mut self, _: ()) {
        self.0 += 1
    }
}

/// Positions (id, x, depth as f32) between two layouts. Ids only in `b` take
/// their `b` position; ids only in `a` are dropped.
pub fn tween(
    a: &[(usize, f32, usize)],
    b: &[(usize, f32, usize)],
    t: f32,
) -> Vec<(usize, f32, f32)> {
    b.iter()
        .map(|&(id, x, d)| match a.iter().find(|p| p.0 == id) {
            Some(&(_, ax, ad)) => (
                id,
                ax + (x - ax) * t,
                ad as f32 + (d as f32 - ad as f32) * t,
            ),
            None => (id, x, d as f32),
        })
        .collect()
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub struct SearchTree {
    tree: Tree,
    balance: bool,
    /// Keys inserted, minus deleted, in order, for rebuilding.
    history: Vec<i32>,
    frames: Vec<Frame>,
    player: Player<FrameIndex>,
    /// Seconds spent on the current frame, for tweening.
    hold: f32,
    /// The frame `hold` was counted for.
    held: usize,
    key: i32,
    seed: u64,
}

impl Default for SearchTree {
    fn default() -> Self {
        let mut s = Self {
            tree: Tree::default(),
            balance: true,
            history: Vec::new(),
            frames: Vec::new(),
            player: Player::new(FrameIndex(0), Vec::new(), SPEED),
            hold: 0.0,
            held: 0,
            key: 25,
            seed: 1,
        };
        let mut scratch = Vec::new();
        for k in START_KEYS {
            if s.tree.insert(k, true, &mut scratch) {
                s.history.push(k);
            }
        }
        s.show_final();
        s
    }
}

impl SearchTree {
    fn snapshot(&self, caption: &str) -> Frame {
        Frame {
            tree: self.tree.clone(),
            focus: None,
            rotation: None,
            caption: caption.to_owned(),
        }
    }

    /// Show only the current tree, with nothing to replay.
    fn show_final(&mut self) {
        self.frames = vec![self.snapshot("")];
        self.player.reset(FrameIndex(0), Vec::new());
        self.hold = 0.0;
        self.held = 0;
    }

    /// Replay `frames`, starting from the first.
    fn play(&mut self, frames: Vec<Frame>) {
        let ops = vec![(); frames.len().saturating_sub(1)];
        self.frames = frames;
        self.player.reset(FrameIndex(0), ops);
        self.hold = 0.0;
        self.held = 0;
    }

    /// Rebuild the tree from `history` with balancing `on` or off.
    pub fn set_balance(&mut self, on: bool) {
        self.balance = on;
        self.tree = Tree::default();
        let mut scratch = Vec::new();
        for &k in &self.history {
            self.tree.insert(k, on, &mut scratch);
        }
        self.show_final();
    }

    fn insert(&mut self) {
        let mut frames = vec![self.snapshot("start")];
        if self.tree.insert(self.key, self.balance, &mut frames) {
            self.history.push(self.key);
        }
        self.play(frames);
    }

    fn delete(&mut self) {
        let mut frames = vec![self.snapshot("start")];
        if self.tree.delete(self.key, self.balance, &mut frames) {
            self.history.retain(|&k| k != self.key);
        }
        self.play(frames);
    }

    fn find(&mut self) {
        let key = self.key;
        let mut frames = vec![self.snapshot("start")];
        let mut at = self.tree.root();
        let mut found = false;
        while let Some(id) = at {
            let Some(n) = self.tree.node(id) else { break };
            let (text, next) = if key == n.key {
                found = true;
                (format!("find {key}: {key} = {}, found", n.key), None)
            } else if key < n.key {
                (format!("find {key}: {key} < {}, go left", n.key), n.left)
            } else {
                (format!("find {key}: {key} > {}, go right", n.key), n.right)
            };
            frames.push(Frame {
                focus: Some(id),
                caption: text,
                ..self.snapshot("")
            });
            at = next;
        }
        if !found {
            frames.push(self.snapshot(&format!("find {key}: not found")));
        }
        self.play(frames);
    }

    fn traverse(&mut self, name: &str, order: Vec<usize>) {
        let mut frames = vec![self.snapshot("start")];
        let mut text = format!("{name}:");
        for id in order {
            let Some(n) = self.tree.node(id) else {
                continue;
            };
            text.push_str(&format!(" {}", n.key));
            frames.push(Frame {
                focus: Some(id),
                caption: text.clone(),
                ..self.snapshot("")
            });
        }
        self.play(frames);
    }

    fn random(&mut self) {
        self.seed += 1;
        let mut rng = Rng::new(self.seed);
        self.tree = Tree::default();
        self.history.clear();
        let mut frames = vec![self.snapshot("start")];
        while self.tree.len() < 15 {
            let key = rng.index(199) as i32 - 99;
            if self.tree.insert(key, self.balance, &mut frames) {
                self.history.push(key);
            }
        }
        self.play(frames);
    }

    fn clear(&mut self) {
        self.tree = Tree::default();
        self.history.clear();
        self.show_final();
    }

    fn tree_height(tree: &Tree) -> i32 {
        tree.root()
            .and_then(|r| tree.node(r))
            .map_or(0, |n| n.height)
    }

    fn shown(&self) -> usize {
        self.player.state.0.min(self.frames.len().saturating_sub(1))
    }
}

impl Formula for SearchTree {
    fn id(&self) -> &'static str {
        "binary-search-tree"
    }

    fn name(&self) -> &'static str {
        "Binary search tree"
    }

    fn category(&self) -> &'static str {
        super::super::GRAPHS
    }

    fn summary(&self) -> &'static str {
        "keeps smaller keys on the left and larger on the right, so a search discards half \
         the tree at every step; AVL rotations keep it from leaning"
    }

    fn notes(&self) -> &'static str {
        "Ordered maps and sets, database indexes, interval trees and the Linux scheduler's \
         red-black tree."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("left subtree < key < right subtree"),
            MathText::new().t("bf = h(left) − h(right),   |bf| ≤ 1"),
            MathText::new().t("h = 1 + max(h(left), h(right))"),
        ]
    }

    fn params(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("n", self.tree.len() as f64),
            ("height", Self::tree_height(&self.tree) as f64),
        ]
    }

    fn stage_height(&self, width: f32) -> f32 {
        (width * 0.62).max(260.0)
    }

    fn min_size(&self) -> egui::Vec2 {
        vec2(420.0, self.stage_height(420.0))
    }

    fn is_animating(&self) -> bool {
        self.player.playing || self.hold * self.player.speed < 1.0
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        self.player.advance(dt);
        let at = self.shown();
        if at != self.held {
            self.held = at;
            self.hold = 0.0;
        }
        self.hold += dt;

        let painter = painter.with_clip_rect(rect);
        let frame = &self.frames[at];
        let prev = &self.frames[at.saturating_sub(1)];
        let t = smoothstep(self.hold * self.player.speed);
        let pos = tween(&prev.tree.layout(), &frame.tree.layout(), t);

        painter.text(
            rect.min + vec2(4.0, HEADER / 2.0),
            Align2::LEFT_CENTER,
            format!(
                "n = {}, height = {}",
                frame.tree.len(),
                Self::tree_height(&frame.tree)
            ),
            FontId::proportional(13.0),
            palette.ink,
        );

        let area = Rect::from_min_max(rect.min + vec2(0.0, HEADER), rect.max - vec2(0.0, FOOTER));
        let count = pos.len().max(1) as f32;
        let radius = RADIUS.min((area.width() - 8.0) / (2.2 * count)).max(5.0);
        let levels = pos
            .iter()
            .fold(1.0_f32, |m, p| m.max(p.2 + 1.0))
            .max(Self::tree_height(&prev.tree) as f32);
        let top = area.min.y + radius + 14.0;
        let span = (area.height() - 2.0 * radius - 20.0).max(1.0);
        let dy = (span / (levels - 1.0).max(1.0)).min(64.0);
        let place = |&(_, x, d): &(usize, f32, f32)| {
            pos2(
                area.min.x + radius + 4.0 + x * (area.width() - 2.0 * radius - 8.0),
                top + d * dy,
            )
        };
        let at_id = |id: usize| pos.iter().find(|p| p.0 == id).map(place);

        let rule = Stroke::new(1.5, palette.rule);
        for p in &pos {
            let Some(n) = frame.tree.node(p.0) else {
                continue;
            };
            for child in [n.left, n.right].into_iter().flatten() {
                if let Some(c) = at_id(child) {
                    painter.line_segment([place(p), c], rule);
                }
            }
        }

        let pivot = frame.rotation.map(|r| match r {
            Rotation::Left(id) | Rotation::Right(id) => id,
        });
        let font = if radius >= 11.0 { 12.0 } else { radius * 1.05 };
        for p in &pos {
            let c = place(p);
            let focused = frame.focus == Some(p.0);
            let (fill, text) = if focused {
                (palette.trace, palette.paper)
            } else {
                (palette.card, palette.ink)
            };
            painter.circle_filled(c, radius, fill);
            painter.circle_stroke(c, radius, Stroke::new(1.5, palette.ink));
            if pivot == Some(p.0) {
                painter.circle_stroke(c, radius + 4.0, Stroke::new(2.5, palette.y));
            }
            if let Some(n) = frame.tree.node(p.0) {
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    n.key.to_string(),
                    FontId::proportional(font),
                    text,
                );
                if self.balance {
                    let bf = frame.tree.balance_factor(p.0);
                    painter.text(
                        c - vec2(0.0, radius + 7.0),
                        Align2::CENTER_CENTER,
                        bf.to_string(),
                        FontId::proportional(9.0),
                        if bf.abs() >= 2 {
                            palette.y
                        } else {
                            palette.muted
                        },
                    );
                }
            }
        }

        painter.text(
            pos2(rect.min.x + 4.0, rect.max.y - FOOTER / 2.0),
            Align2::LEFT_CENTER,
            &frame.caption,
            FontId::proportional(13.0),
            palette.ink,
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Key");
            ui.add(egui::DragValue::new(&mut self.key).range(-99..=99));
            if ui.button("Insert").clicked() {
                self.insert();
            }
            if ui.button("Delete").clicked() {
                self.delete();
            }
            if ui.button("Find").clicked() {
                self.find();
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Random 15").clicked() {
                self.random();
            }
            if ui.button("Clear").clicked() {
                self.clear();
            }
            let mut on = self.balance;
            if ui.checkbox(&mut on, "Balance (AVL)").changed() {
                self.set_balance(on);
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("In-order").clicked() {
                let order = self.tree.inorder();
                self.traverse("in-order", order);
            }
            if ui.button("Pre-order").clicked() {
                let order = self.tree.preorder();
                self.traverse("pre-order", order);
            }
            if ui.button("Post-order").clicked() {
                let order = self.tree.postorder();
                self.traverse("post-order", order);
            }
        });

        ui.add_space(10.0);
        replay::transport(ui, &mut self.player, 20.0);

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "Filled node: the one being compared or visited. Amber ring: the node a rotation \
                 moves down. The small number above a node is its balance factor; it turns \
                 amber at 2 or more. Space, Left and Right step",
            )
            .small()
            .color(palette.muted),
        );
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        replay::transport_keys(ctx, &mut self.player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_moves_shared_ids_and_shows_new_ones_in_place() {
        let a = vec![(0, 0.2, 0), (1, 0.6, 1)];
        let b = vec![(0, 0.4, 1), (2, 0.9, 2)];
        let mid = tween(&a, &b, 0.5);
        assert_eq!(mid.len(), 2);
        let p0 = mid.iter().find(|p| p.0 == 0).unwrap();
        assert!((p0.1 - 0.3).abs() < 1e-6 && (p0.2 - 0.5).abs() < 1e-6);
        let p2 = mid.iter().find(|p| p.0 == 2).unwrap();
        assert_eq!((p2.1, p2.2), (0.9, 2.0));
    }

    #[test]
    fn default_tree_is_built_and_balanced() {
        let t = SearchTree::default();
        assert_eq!(t.tree.len(), 10);
        assert!(t.player.done());
        assert!(
            t.tree
                .inorder()
                .iter()
                .all(|&id| t.tree.balance_factor(id).abs() <= 1)
        );
    }

    #[test]
    fn toggling_balance_rebuilds_from_the_history() {
        let mut t = SearchTree::default();
        t.set_balance(false);
        assert_eq!(t.tree.keys(), {
            let mut k = t.history.clone();
            k.sort();
            k
        });
        assert_eq!(t.tree.len(), 10);
    }
}
