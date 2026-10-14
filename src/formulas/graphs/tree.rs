//! A binary search tree with optional AVL balancing. Every comparison and
//! rotation records a snapshot frame, so a viewport can replay the operation.
//! Nodes live in an arena and ids are never reused, so frames can tween a
//! node's position by id even as rotations move it.

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub key: i32,
    pub left: Option<usize>,
    pub right: Option<usize>,
    /// A leaf has height 1.
    pub height: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tree {
    nodes: Vec<Option<Node>>,
    root: Option<usize>,
}

/// The id is the node that moves down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Left(usize),
    Right(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub tree: Tree,
    pub focus: Option<usize>,
    pub rotation: Option<Rotation>,
    pub caption: String,
}

/// Where a subtree hangs, so a frame can show it attached mid-operation.
#[derive(Clone, Copy)]
enum Link {
    Root,
    Left(usize),
    Right(usize),
}

impl Tree {
    pub fn root(&self) -> Option<usize> {
        self.root
    }

    pub fn node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id).and_then(Option::as_ref)
    }

    pub fn len(&self) -> usize {
        self.nodes.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn contains(&self, key: i32) -> bool {
        let mut at = self.root;
        while let Some(n) = at.and_then(|id| self.node(id)) {
            if key == n.key {
                return true;
            }
            at = if key < n.key { n.left } else { n.right };
        }
        false
    }

    fn height(&self, id: Option<usize>) -> i32 {
        id.and_then(|id| self.node(id)).map_or(0, |n| n.height)
    }

    /// height(left) - height(right).
    pub fn balance_factor(&self, id: usize) -> i32 {
        self.node(id)
            .map_or(0, |n| self.height(n.left) - self.height(n.right))
    }

    fn n(&self, id: usize) -> &Node {
        self.nodes[id].as_ref().expect("live node id")
    }

    fn n_mut(&mut self, id: usize) -> &mut Node {
        self.nodes[id].as_mut().expect("live node id")
    }

    fn update_height(&mut self, id: usize) {
        let n = self.n(id);
        let h = 1 + self.height(n.left).max(self.height(n.right));
        self.n_mut(id).height = h;
    }

    /// Recompute every cached height from the links. Mid-operation the
    /// ancestors of a change are stale, so snapshots are normalised.
    fn fix_heights(&mut self) {
        fn go(t: &mut Tree, at: Option<usize>) -> i32 {
            let Some(id) = at else { return 0 };
            let (l, r) = (t.n(id).left, t.n(id).right);
            let h = 1 + go(t, l).max(go(t, r));
            t.n_mut(id).height = h;
            h
        }
        go(self, self.root);
    }

    fn set_link(&mut self, link: Link, to: usize) {
        match link {
            Link::Root => self.root = Some(to),
            Link::Left(p) => self.n_mut(p).left = Some(to),
            Link::Right(p) => self.n_mut(p).right = Some(to),
        }
    }

    fn snap(
        &self,
        frames: &mut Vec<Frame>,
        focus: Option<usize>,
        rotation: Option<Rotation>,
        caption: String,
    ) {
        let mut tree = self.clone();
        tree.fix_heights();
        frames.push(Frame {
            tree,
            focus,
            rotation,
            caption,
        });
    }

    /// Like `snap`, but with `to` attached at `link` in the snapshot only.
    fn snap_linked(&self, frames: &mut Vec<Frame>, link: Link, to: usize, caption: String) {
        let mut tree = self.clone();
        tree.set_link(link, to);
        tree.fix_heights();
        frames.push(Frame {
            tree,
            focus: Some(to),
            rotation: None,
            caption,
        });
    }

    /// Insert `key`; returns false (and changes nothing) if already present.
    pub fn insert(&mut self, key: i32, balance: bool, frames: &mut Vec<Frame>) -> bool {
        let (root, inserted) = self.insert_at(self.root, key, balance, frames, Link::Root);
        self.root = root;
        if root.is_some() && inserted && self.len() == 1 {
            self.snap(frames, root, None, format!("insert {key}: new leaf"));
        }
        inserted
    }

    /// Delete `key`; returns false (and changes nothing) if absent.
    pub fn delete(&mut self, key: i32, balance: bool, frames: &mut Vec<Frame>) -> bool {
        let (root, deleted) = self.delete_at(self.root, key, balance, frames, Link::Root);
        self.root = root;
        if deleted {
            self.snap(frames, None, None, format!("delete {key}: done"));
        }
        deleted
    }

    fn insert_at(
        &mut self,
        at: Option<usize>,
        key: i32,
        balance: bool,
        frames: &mut Vec<Frame>,
        link: Link,
    ) -> (Option<usize>, bool) {
        let Some(id) = at else {
            // The parent attaches the leaf and records the frame.
            self.nodes.push(Some(Node {
                key,
                left: None,
                right: None,
                height: 1,
            }));
            return (Some(self.nodes.len() - 1), true);
        };
        let here = self.n(id).key;
        if key == here {
            let caption = format!("insert {key}: {key} is already in the tree");
            self.snap(frames, Some(id), None, caption);
            return (at, false);
        }
        let go_left = key < here;
        let (cmp, side) = if go_left {
            ('<', "left")
        } else {
            ('>', "right")
        };
        let caption = format!("insert {key}: {key} {cmp} {here}, go {side}");
        self.snap(frames, Some(id), None, caption);
        let child = if go_left {
            self.n(id).left
        } else {
            self.n(id).right
        };
        let child_link = if go_left {
            Link::Left(id)
        } else {
            Link::Right(id)
        };
        let (new_child, inserted) = self.insert_at(child, key, balance, frames, child_link);
        if let Some(c) = new_child {
            self.set_link(child_link, c);
        }
        if !inserted {
            return (at, false);
        }
        if child.is_none() {
            let caption = format!("insert {key}: new leaf");
            self.snap(frames, new_child, None, caption);
        }
        self.update_height(id);
        let id = if balance {
            self.rebalance(id, link, frames)
        } else {
            id
        };
        (Some(id), true)
    }

    fn delete_at(
        &mut self,
        at: Option<usize>,
        key: i32,
        balance: bool,
        frames: &mut Vec<Frame>,
        link: Link,
    ) -> (Option<usize>, bool) {
        let Some(id) = at else {
            self.snap(frames, None, None, format!("delete {key}: not found"));
            return (None, false);
        };
        let here = self.n(id).key;
        if key == here {
            let (left, right) = (self.n(id).left, self.n(id).right);
            match (left, right) {
                (Some(_), Some(r)) => {
                    let mut s = r;
                    while let Some(next) = self.n(s).left {
                        s = next;
                    }
                    let succ = self.n(s).key;
                    let caption = format!("delete {key}: two children, copy successor {succ}");
                    self.snap(frames, Some(id), None, caption);
                    self.n_mut(id).key = succ;
                    let (new_right, _) =
                        self.delete_at(Some(r), succ, balance, frames, Link::Right(id));
                    self.n_mut(id).right = new_right;
                }
                _ => {
                    let what = if left.is_none() && right.is_none() {
                        "leaf, remove it"
                    } else {
                        "one child, replace it with the child"
                    };
                    self.snap(frames, Some(id), None, format!("delete {key}: {what}"));
                    self.nodes[id] = None;
                    return (left.or(right), true);
                }
            }
        } else {
            let go_left = key < here;
            let (cmp, side) = if go_left {
                ('<', "left")
            } else {
                ('>', "right")
            };
            let caption = format!("delete {key}: {key} {cmp} {here}, go {side}");
            self.snap(frames, Some(id), None, caption);
            let (child, child_link) = if go_left {
                (self.n(id).left, Link::Left(id))
            } else {
                (self.n(id).right, Link::Right(id))
            };
            let (new_child, deleted) = self.delete_at(child, key, balance, frames, child_link);
            if !deleted {
                return (at, false);
            }
            if go_left {
                self.n_mut(id).left = new_child;
            } else {
                self.n_mut(id).right = new_child;
            }
        }
        self.update_height(id);
        let id = if balance {
            self.rebalance(id, link, frames)
        } else {
            id
        };
        (Some(id), true)
    }

    /// Restore |balance factor| <= 1 at `id`, returning the new subtree root.
    fn rebalance(&mut self, id: usize, link: Link, frames: &mut Vec<Frame>) -> usize {
        self.update_height(id);
        let bf = self.balance_factor(id);
        if bf.abs() <= 1 {
            return id;
        }
        let key = self.n(id).key;
        let left_heavy = bf > 1;
        let child = if left_heavy {
            self.n(id).left
        } else {
            self.n(id).right
        }
        .expect("a heavy side has a child");
        let cbf = self.balance_factor(child);
        let zigzag = if left_heavy { cbf < 0 } else { cbf > 0 };
        if zigzag {
            let ckey = self.n(child).key;
            let (rot, dir) = if left_heavy {
                (Rotation::Left(child), "left")
            } else {
                (Rotation::Right(child), "right")
            };
            let caption = format!("balance factor {bf} at {key}: rotate {dir} at {ckey} first");
            self.snap(frames, Some(child), Some(rot), caption);
            let new_child = if left_heavy {
                self.rotate_left(child)
            } else {
                self.rotate_right(child)
            };
            let child_link = if left_heavy {
                Link::Left(id)
            } else {
                Link::Right(id)
            };
            self.set_link(child_link, new_child);
            let caption = format!("rotated {dir} at {ckey}");
            self.snap(frames, Some(new_child), None, caption);
        }
        let (rot, dir) = if left_heavy {
            (Rotation::Right(id), "right")
        } else {
            (Rotation::Left(id), "left")
        };
        let caption = format!("balance factor {bf} at {key}: rotate {dir}");
        self.snap(frames, Some(id), Some(rot), caption);
        let new_root = if left_heavy {
            self.rotate_right(id)
        } else {
            self.rotate_left(id)
        };
        self.snap_linked(frames, link, new_root, format!("rotated {dir} at {key}"));
        new_root
    }

    /// The right child rises; returns it. The caller relinks the parent.
    fn rotate_left(&mut self, id: usize) -> usize {
        let r = self.n(id).right.expect("rotate_left needs a right child");
        let rl = self.n(r).left;
        self.n_mut(id).right = rl;
        self.n_mut(r).left = Some(id);
        self.update_height(id);
        self.update_height(r);
        r
    }

    /// The left child rises; returns it. The caller relinks the parent.
    fn rotate_right(&mut self, id: usize) -> usize {
        let l = self.n(id).left.expect("rotate_right needs a left child");
        let lr = self.n(l).right;
        self.n_mut(id).left = lr;
        self.n_mut(l).right = Some(id);
        self.update_height(id);
        self.update_height(l);
        l
    }

    pub fn inorder(&self) -> Vec<usize> {
        let mut out = Vec::new();
        self.walk(self.root, 0, &mut |_, _| {}, &mut out, Order::In);
        out
    }

    pub fn preorder(&self) -> Vec<usize> {
        let mut out = Vec::new();
        self.walk(self.root, 0, &mut |_, _| {}, &mut out, Order::Pre);
        out
    }

    pub fn postorder(&self) -> Vec<usize> {
        let mut out = Vec::new();
        self.walk(self.root, 0, &mut |_, _| {}, &mut out, Order::Post);
        out
    }

    fn walk(
        &self,
        at: Option<usize>,
        depth: usize,
        visit: &mut dyn FnMut(usize, usize),
        out: &mut Vec<usize>,
        order: Order,
    ) {
        let Some(id) = at else { return };
        let n = self.n(id);
        if order == Order::Pre {
            out.push(id);
            visit(id, depth);
        }
        self.walk(n.left, depth + 1, visit, out, order);
        if order == Order::In {
            out.push(id);
            visit(id, depth);
        }
        self.walk(n.right, depth + 1, visit, out, order);
        if order == Order::Post {
            out.push(id);
            visit(id, depth);
        }
    }

    /// Keys in ascending order.
    pub fn keys(&self) -> Vec<i32> {
        self.inorder().iter().map(|&id| self.n(id).key).collect()
    }

    /// (id, x in 0..1 by in-order rank, depth) for every node.
    pub fn layout(&self) -> Vec<(usize, f32, usize)> {
        let mut depths = vec![0; self.nodes.len()];
        let mut scratch = Vec::new();
        self.walk(
            self.root,
            0,
            &mut |id, d| depths[id] = d,
            &mut scratch,
            Order::In,
        );
        let ids = self.inorder();
        let len = ids.len() as f32;
        ids.iter()
            .enumerate()
            .map(|(rank, &id)| (id, (rank as f32 + 0.5) / len, depths[id]))
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Order {
    Pre,
    In,
    Post,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    fn build(keys: &[i32], balance: bool) -> (Tree, Vec<Frame>) {
        let mut t = Tree::default();
        let mut frames = Vec::new();
        for &k in keys {
            t.insert(k, balance, &mut frames);
        }
        (t, frames)
    }

    fn rotations(frames: &[Frame]) -> Vec<&'static str> {
        frames
            .iter()
            .filter_map(|f| match f.rotation {
                Some(Rotation::Left(_)) => Some("L"),
                Some(Rotation::Right(_)) => Some("R"),
                None => None,
            })
            .collect()
    }

    fn check_invariants(t: &Tree, balance: bool) {
        let keys = t.keys();
        assert!(
            keys.windows(2).all(|w| w[0] < w[1]),
            "in-order is sorted: {keys:?}"
        );
        for id in t.inorder() {
            let n = t.node(id).unwrap();
            let h = |c: Option<usize>| c.and_then(|c| t.node(c)).map_or(0, |n| n.height);
            assert_eq!(
                n.height,
                1 + h(n.left).max(h(n.right)),
                "cached height of {}",
                n.key
            );
            if balance {
                assert!(t.balance_factor(id).abs() <= 1, "AVL balance at {}", n.key);
            }
        }
    }

    #[test]
    fn the_four_avl_cases_rotate_into_the_textbook_shape() {
        for (keys, want) in [
            ([3, 2, 1], vec!["R"]),
            ([1, 2, 3], vec!["L"]),
            ([3, 1, 2], vec!["L", "R"]),
            ([1, 3, 2], vec!["R", "L"]),
        ] {
            let (t, frames) = build(&keys, true);
            assert_eq!(rotations(&frames), want, "{keys:?}");
            let root = t.node(t.root().unwrap()).unwrap();
            assert_eq!(root.key, 2, "{keys:?}");
            assert_eq!(t.node(root.left.unwrap()).unwrap().key, 1);
            assert_eq!(t.node(root.right.unwrap()).unwrap().key, 3);
        }
    }

    #[test]
    fn without_balancing_sorted_input_makes_a_chain() {
        let (t, frames) = build(&[1, 2, 3, 4, 5], false);
        assert!(rotations(&frames).is_empty());
        assert_eq!(t.node(t.root().unwrap()).unwrap().height, 5);
    }

    #[test]
    fn random_inserts_and_deletes_keep_the_invariants() {
        for balance in [false, true] {
            let mut rng = Rng::new(42);
            let mut t = Tree::default();
            let mut frames = Vec::new();
            for _ in 0..500 {
                let k = rng.index(60) as i32;
                if rng.f32() < 0.6 {
                    t.insert(k, balance, &mut frames);
                } else {
                    t.delete(k, balance, &mut frames);
                }
                check_invariants(&t, balance);
            }
        }
    }

    #[test]
    fn missing_keys_duplicates_and_empty_trees_leave_one_frame_and_no_change() {
        let mut t = Tree::default();
        let mut frames = Vec::new();
        assert!(!t.delete(5, true, &mut frames));
        assert_eq!(frames.len(), 1);
        assert!(frames[0].caption.contains("not found"));
        let (mut t, _) = build(&[5, 3, 8], true);
        let before = t.clone();
        let mut frames = Vec::new();
        assert!(!t.insert(3, true, &mut frames));
        assert!(
            frames
                .last()
                .unwrap()
                .caption
                .contains("already in the tree")
        );
        assert!(!t.delete(4, true, &mut frames));
        assert!(frames.last().unwrap().caption.contains("not found"));
        assert_eq!(t, before);
    }

    #[test]
    fn deleting_a_node_with_two_children_uses_the_successor() {
        let (mut t, _) = build(&[50, 30, 70, 60, 80], true);
        let mut frames = Vec::new();
        assert!(t.delete(50, true, &mut frames));
        assert_eq!(t.keys(), vec![30, 60, 70, 80]);
        assert!(frames.iter().any(|f| f.caption.contains("successor 60")));
    }

    #[test]
    fn traversals_visit_in_their_orders() {
        let (t, _) = build(&[2, 1, 3], true);
        let keys = |ids: Vec<usize>| {
            ids.iter()
                .map(|&i| t.node(i).unwrap().key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(t.inorder()), vec![1, 2, 3]);
        assert_eq!(keys(t.preorder()), vec![2, 1, 3]);
        assert_eq!(keys(t.postorder()), vec![1, 3, 2]);
    }

    #[test]
    fn layout_spreads_nodes_by_rank_and_depth() {
        let (t, _) = build(&[2, 1, 3], true);
        let mut l = t.layout();
        l.sort_by(|a, b| a.1.total_cmp(&b.1));
        let depths: Vec<usize> = l.iter().map(|e| e.2).collect();
        assert_eq!(depths, vec![1, 0, 1]);
        assert!(l.iter().all(|e| e.1 > 0.0 && e.1 < 1.0));
    }

    #[test]
    fn every_frame_is_a_consistent_snapshot() {
        fn heights_ok(t: &Tree, at: Option<usize>) -> i32 {
            let Some(n) = at.and_then(|id| t.node(id)) else {
                return 0;
            };
            let h = 1 + heights_ok(t, n.left).max(heights_ok(t, n.right));
            assert_eq!(n.height, h, "cached height of {}", n.key);
            h
        }
        for balance in [false, true] {
            let mut rng = Rng::new(7);
            let mut t = Tree::default();
            for _ in 0..300 {
                let k = rng.index(40) as i32;
                let mut frames = Vec::new();
                if rng.f32() < 0.6 {
                    t.insert(k, balance, &mut frames);
                } else {
                    t.delete(k, balance, &mut frames);
                }
                for f in &frames {
                    heights_ok(&f.tree, f.tree.root());
                }
                assert_eq!(&frames.last().unwrap().tree, &t, "last frame is the result");
            }
        }
    }

    #[test]
    fn node_ids_survive_rotations() {
        let (t1, _) = build(&[1, 2], true);
        let id_of_2 = t1.inorder()[1];
        let (t2, _) = build(&[1, 2, 3], true);
        assert_eq!(
            t2.node(id_of_2).unwrap().key,
            2,
            "same id, same key after rotating"
        );
    }
}
