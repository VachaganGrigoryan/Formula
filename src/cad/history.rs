//! Undo and redo.
//!
//! Each [`Edit`] knows how to undo itself, and undoing one produces the edit
//! that redoes it. A removed viewport keeps its live formula, so undoing a
//! delete brings it back exactly as it was.

use egui::Rect;

use super::board::{Board, Viewport};

pub enum Edit {
    /// A viewport was placed.
    Added { uid: u64 },
    /// A viewport was deleted from z position `index`.
    Removed { index: usize, vp: Box<Viewport> },
    /// Viewports were moved or resized: (uid, before, after).
    Reshaped(Vec<(u64, Rect, Rect)>),
    Renamed {
        uid: u64,
        before: String,
        after: String,
    },
    /// Several edits made as one step (e.g. deleting a selection).
    Group(Vec<Edit>),
}

impl Edit {
    /// Undo this edit on the board, returning the edit that redoes it.
    fn invert(self, board: &mut Board) -> Edit {
        match self {
            Edit::Added { uid } => match board.remove(uid) {
                Some((index, vp)) => Edit::Removed {
                    index,
                    vp: Box::new(vp),
                },
                None => Edit::Group(Vec::new()),
            },
            Edit::Removed { index, vp } => {
                let uid = vp.uid;
                board.insert(index, *vp);
                Edit::Added { uid }
            }
            Edit::Reshaped(changes) => {
                for &(uid, before, _) in &changes {
                    if let Some(vp) = board.get_mut(uid) {
                        vp.rect = before;
                    }
                }
                Edit::Reshaped(changes.into_iter().map(|(u, b, a)| (u, a, b)).collect())
            }
            Edit::Renamed { uid, before, after } => {
                if let Some(vp) = board.get_mut(uid) {
                    vp.title = before.clone();
                }
                Edit::Renamed {
                    uid,
                    before: after,
                    after: before,
                }
            }
            // Undo last-first. The inverses come out last-first too, so
            // inverting the result again replays them first-last.
            Edit::Group(edits) => {
                Edit::Group(edits.into_iter().rev().map(|e| e.invert(board)).collect())
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Edit::Added { .. } => "place".into(),
            Edit::Removed { vp, .. } => format!("delete {}", vp.title),
            Edit::Reshaped(c) if c.len() == 1 => "move/resize".into(),
            Edit::Reshaped(c) => format!("move {} viewports", c.len()),
            Edit::Renamed { after, .. } => format!("rename to {after}"),
            Edit::Group(g) => format!("{} changes", g.len()),
        }
    }
}

#[derive(Default)]
pub struct History {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

/// How many steps are kept.
const LIMIT: usize = 200;

impl History {
    /// Record an edit that has already been applied to the board.
    pub fn push(&mut self, edit: Edit) {
        if matches!(&edit, Edit::Group(g) if g.is_empty()) {
            return;
        }
        if let Edit::Reshaped(c) = &edit
            && c.iter().all(|(_, b, a)| b == a)
        {
            return;
        }
        self.undo.push(edit);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, board: &mut Board) -> Option<String> {
        let edit = self.undo.pop()?;
        let what = edit.describe();
        let redo = edit.invert(board);
        self.redo.push(redo);
        Some(what)
    }

    pub fn redo(&mut self, board: &mut Board) -> Option<String> {
        let edit = self.redo.pop()?;
        let undo = edit.invert(board);
        let what = undo.describe();
        self.undo.push(undo);
        Some(what)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas;
    use egui::{Vec2, pos2, vec2};

    fn board_with_two() -> (Board, u64, u64) {
        let mut b = Board::default();
        let lerp = formulas::tool("lerp").unwrap();
        let x = b.add(lerp, pos2(0.0, 0.0), Vec2::ZERO, 0.0);
        let y = b.add(lerp, pos2(800.0, 0.0), Vec2::ZERO, 0.0);
        (b, x, y)
    }

    #[test]
    fn undo_and_redo_a_placement() {
        let (mut b, _, y) = board_with_two();
        let mut h = History::default();
        h.push(Edit::Added { uid: y });
        h.undo(&mut b);
        assert!(b.get(y).is_none());
        h.redo(&mut b);
        assert!(b.get(y).is_some());
        assert!(h.can_undo() && !h.can_redo());
    }

    #[test]
    fn undo_delete_restores_position_and_state() {
        let (mut b, x, y) = board_with_two();
        let mut h = History::default();
        b.get_mut(x).unwrap().title = "Mine".into();
        let edits = [x, y]
            .iter()
            .map(|&u| {
                let (index, vp) = b.remove(u).unwrap();
                Edit::Removed {
                    index,
                    vp: Box::new(vp),
                }
            })
            .collect();
        h.push(Edit::Group(edits));
        assert!(b.viewports.is_empty());
        h.undo(&mut b);
        assert_eq!(
            b.viewports.iter().map(|v| v.uid).collect::<Vec<_>>(),
            vec![x, y]
        );
        assert_eq!(b.get(x).unwrap().title, "Mine");
        h.redo(&mut b);
        assert!(b.viewports.is_empty());
        h.undo(&mut b);
        assert_eq!(b.viewports.len(), 2);
    }

    #[test]
    fn reshape_and_rename() {
        let (mut b, x, _) = board_with_two();
        let mut h = History::default();
        let before = b.get(x).unwrap().rect;
        let after = before.translate(vec2(50.0, 25.0));
        b.get_mut(x).unwrap().rect = after;
        h.push(Edit::Reshaped(vec![(x, before, after)]));
        b.get_mut(x).unwrap().title = "B".into();
        h.push(Edit::Renamed {
            uid: x,
            before: "VP-01".into(),
            after: "B".into(),
        });
        h.undo(&mut b);
        assert_eq!(b.get(x).unwrap().title, "VP-01");
        h.undo(&mut b);
        assert_eq!(b.get(x).unwrap().rect, before);
        h.redo(&mut b);
        assert_eq!(b.get(x).unwrap().rect, after);
        // A no-op reshape is not recorded.
        h.push(Edit::Reshaped(vec![(x, after, after)]));
        h.redo(&mut b);
        assert_eq!(b.get(x).unwrap().title, "B");
    }
}
