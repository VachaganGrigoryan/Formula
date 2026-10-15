# Graphs & Trees Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "Graphs & Trees" category with seven step-through algorithm tools: BFS, DFS, Dijkstra and A* pathfinding on a drawable grid, a binary search tree with optional AVL balancing, and Kruskal's and Prim's minimum spanning trees.

**Architecture:**
- First, pull the sorting tools' record-then-replay player out into a generic `formulas::replay::Player<S: Replay>`, without changing behavior.
- Each new algorithm runs once, UI-free. It records a log of operations (pathfinding, MST) or snapshots (trees), and its viewport replays the log with the shared transport controls and keys.
- Viewports of the same family share one grid or graph per board through `ToolEnv`, exactly as sorting tools share one list. That lets algorithms be compared on the same input.

**Tech Stack:** Rust 2024, egui/eframe 0.36, the existing `Formula`/`TOOLS` architecture. No new dependencies.

**Spec:** Scope 2 of the roadmap, `/Users/vachagan/.claude/plans/use-to-create-plan-magical-dusk.md` (phases 2.1 to 2.4). Read `CLAUDE.md` at the repo root first. It lists every check a new tool must pass.

## Global Constraints

- **Gate before every commit.** This script stops at the first failing step: `/private/tmp/claude-501/-Users-vachagan-WS-Formula/18c52e04-e684-493b-87ae-26c6ff992bda/scratchpad/gate.sh` (fmt check, clippy `-D warnings`, `cargo test`). It must print `GATE OK`.
- **Every new tool needs** all of the following:
  - a `TOOLS` entry with a unique 3-letter alias (BFS, DFS, DIJ, AST, BST, KRU, PRI);
  - an icon arm `"<id>" => { … }` in `src/cad/icons.rs`;
  - `assets/content/<id>/{doc.md, main.py, main.ts, main.rs}`;
  - a `doc.md` starting with `# <Tool name>`, with exactly the ten `## ` headings of `content::SECTIONS` in order;
  - ASCII-only code files;
  - UI text that egui's bundled font can draw (no ′ ∈ → ᵢ ᵣ, no combining accents).
- **Exports** must pass `/private/tmp/claude-501/-Users-vachagan-WS-Formula/18c52e04-e684-493b-87ae-26c6ff992bda/scratchpad/check-export.sh <tool-id>`, which must print `EXPORTS OK: <tool-id>`. It runs `main.py` headlessly, `tsc --strict` on `main.ts`, and `cargo check` of `main.rs` against macroquad 0.4.
  - Python: numpy and matplotlib only. Keep the animation in a variable named `anim` and end with `plt.show()`.
  - TypeScript: creates its own `<canvas>`.
  - Rust: macroquad 0.4.
  - Mirror the existing exports in `assets/content/matrix-transform/` (header comment, layout, interaction).
- **Colors and structure:**
  - Formulas use `Palette` colors only (`palette.x`, `.y`, `.trace`, `.fill`, `.ink`, `.muted`, `.rule`, `.card`, `.paper`).
  - Math and algorithms live in UI-free modules with unit tests.
  - Randomness only from `crate::rng::Rng` (seeded).
- **Sliders:** a slider with `step_by` must also have `.clamping(egui::SliderClamping::Edits)`. The test `formulas::tests::showing_controls_leaves_parameters_unchanged` enforces this.
- **New category:** `pub const GRAPHS: &str = "Graphs & Trees";` in `src/formulas/mod.rs`, registered after the `SORTING` tools. Add `formulas::GRAPHS => "Graphs",` to `short()` in `src/cad/chrome.rs`, or `every_category_has_a_toolbar_name` fails.
- **Targets:** must build for native and `wasm32-unknown-unknown` (`cargo check --lib --target wasm32-unknown-unknown`).
- **Commits:** one commit per task, ending with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Work on branch `feat/graphs-trees`. Never push.

## Review Focus

1. **Goal walled off, or start right next to the goal.** The search ends with `NoPath` and shows a "No path" caption, or gives a two-cell path. It never panics or loops. The grid never lets start and goal share a cell. Tests: Task 3.
2. **The user moves start or goal onto a wall, onto the other marker, or off the grid.** The move is ignored; walls never cover start or goal. Tests: Task 2.
3. **Deleting a missing key, deleting from an empty tree, or inserting a duplicate.** The operation records a single frame captioned "… not found" or "… is already in the tree", and the tree is unchanged. Tests: Task 6.
4. **Points dragged on top of each other, or into a line, in the MST graph.** The edge list stays finite and the MST still spans every point. Coincident points are nudged apart, so the triangulation never sees duplicates. Tests: Task 8.
5. **The shared grid or graph changes while a viewport is mid-replay.** Every viewport re-records on its next frame, with the cursor clamped to the new log. No stale indices reach the painter. Tests: Tasks 4 and 9.

---

## File Structure

| File | Responsibility |
| --- | --- |
| `src/formulas/replay.rs` (new) | `Replay` trait, `Player<S>`, transport controls and keys shared by every step-through tool |
| `src/formulas/sorting/mod.rs`, `sorting/race.rs` (modify) | Use `Player<State>` instead of their own cursor/speed/carry fields |
| `src/formulas/graphs/mod.rs` (new) | `Session` (shared grid and graph), module declarations, tool constructors |
| `src/formulas/graphs/grid.rs` (new) | `Grid`: walls, mud weights, start and goal, maze and scatter generators |
| `src/formulas/graphs/path.rs` (new) | `Search` (BFS, DFS, Dijkstra, A*), `record()`, `PathOp`, `PathState: Replay` |
| `src/formulas/graphs/pathfinding.rs` (new) | The pathfinding viewport (one type, four tools) |
| `src/formulas/graphs/tree.rs` (new) | Arena BST/AVL with snapshot frames for every comparison and rotation |
| `src/formulas/graphs/bst.rs` (new) | The binary search tree viewport |
| `src/formulas/graphs/mst.rs` (new) | `Graph` (points plus Delaunay edges), `Spanning` (Kruskal, Prim), `MstOp`, `MstState: Replay` |
| `src/formulas/graphs/spanning.rs` (new) | The MST viewport (one type, two tools) |
| `src/formulas/mod.rs` (modify) | `GRAPHS` category, `ToolEnv.graphs`, seven `tool!` entries |
| `src/cad/chrome.rs`, `src/cad/icons.rs` (modify) | Toolbar name and seven icons |
| `assets/content/{bfs,dfs,dijkstra,a-star,binary-search-tree,kruskal,prim}/` (new) | Docs and exports |
| `README.md`, `CLAUDE.md` (modify) | Tool table rows; architecture note on `replay` and `graphs` |

---

### Task 1: Generic replay player (refactor sorting onto it)

**Files:**
- Create: `src/formulas/replay.rs`
- Modify: `src/formulas/mod.rs` (add `pub mod replay;`)
- Modify: `src/formulas/sorting/algorithms.rs` (`impl Replay for State`)
- Modify: `src/formulas/sorting/mod.rs`: `SortVisualizer` fields at lines 317–337, `sync`/`seek`/`done` at 364–397, the advance at the top of `paint` (571–590), `handle_keys` (727–752), and the player part of `controls` (754–830).
- Modify: `src/formulas/sorting/race.rs`: replace `view.ops` / `view.cursor` / `view.state` / `view.playing` with `view.player.*`.

**Interfaces:**
- Produces:
  - `pub trait Replay: Clone { type Op: Copy; fn apply(&mut self, op: Self::Op); }`
  - `pub struct Player<S: Replay> { pub state: S, pub ops: Vec<S::Op>, pub cursor: usize, pub playing: bool, pub speed: f32, initial: S, carry: f32 }`
  - Methods:
    - `Player::new(initial: S, ops: Vec<S::Op>, speed: f32) -> Self`: starts playing at cursor 0.
    - `reset(&mut self, initial: S, ops: Vec<S::Op>)`: keeps `speed`, sets cursor 0 and playing.
    - `seek(&mut self, k: usize)`: clamps to `ops.len()`; going backwards replays from `initial`.
    - `advance(&mut self, dt: f32) -> usize`: steps applied this frame; 0 when paused or done; sets `playing = false` when it reaches the end.
    - `done(&self) -> bool`
    - `toggle(&mut self)`: play/pause, or replay from 0 when done.
  - Free functions: `pub fn transport(ui: &mut egui::Ui, p: &mut Player<impl Replay>, max_speed: f32)` (Speed slider, Step slider, then Restart / Step back / Play·Pause·Replay / Step / Finish buttons) and `pub fn transport_keys<S: Replay>(ctx: &egui::Context, p: &mut Player<S>)` (Space toggles, Left steps back, Right steps forward; each consumed only when used).

- [ ] **Step 1: Write the failing tests** in `src/formulas/replay.rs`. Start the file with the doc comment and a tests module that uses a toy state:

```rust
//! Step-through playback for algorithms that are recorded once and then
//! replayed: a log of operations applied to a state, at any speed, forward
//! or backward (backward replays from the start), like scrubbing a video.

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy state: the sum of the ops applied so far.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Sum(i32);

    impl Replay for Sum {
        type Op = i32;
        fn apply(&mut self, op: i32) {
            self.0 += op;
        }
    }

    fn player() -> Player<Sum> {
        Player::new(Sum(0), vec![1, 2, 3, 4], 2.0)
    }

    #[test]
    fn seeking_backwards_matches_seeking_forwards() {
        let mut p = player();
        p.seek(3);
        assert_eq!(p.state, Sum(6));
        p.seek(1);
        assert_eq!((p.state.clone(), p.cursor), (Sum(1), 1));
        p.seek(99);
        assert_eq!((p.state.clone(), p.cursor), (Sum(10), 4), "clamped to the end");
    }

    #[test]
    fn advance_carries_fractional_steps_between_frames() {
        let mut p = player(); // 2 steps per second
        assert_eq!(p.advance(0.3), 0);
        assert_eq!(p.advance(0.3), 1);
        assert_eq!(p.advance(0.3), 0);
        assert_eq!(p.cursor, 1);
    }

    #[test]
    fn advance_never_overshoots_and_stops_at_the_end() {
        let mut p = player();
        assert_eq!(p.advance(100.0), 4);
        assert!(p.done() && !p.playing);
        assert_eq!(p.advance(1.0), 0);
        assert_eq!(p.state, Sum(10));
    }

    #[test]
    fn paused_players_do_not_move() {
        let mut p = player();
        p.playing = false;
        assert_eq!(p.advance(10.0), 0);
        assert_eq!(p.cursor, 0);
    }

    #[test]
    fn toggle_replays_from_the_start_when_done() {
        let mut p = player();
        p.seek(4);
        p.playing = false;
        p.toggle();
        assert_eq!((p.cursor, p.playing), (0, true));
        p.toggle();
        assert!(!p.playing);
    }

    #[test]
    fn reset_keeps_the_speed() {
        let mut p = player();
        p.speed = 7.0;
        p.seek(2);
        p.reset(Sum(100), vec![5]);
        assert_eq!((p.cursor, p.speed, p.state.clone()), (0, 7.0, Sum(100)));
        assert!(p.playing);
    }

    #[test]
    fn keys_play_pause_and_step() {
        let ctx = egui::Context::default();
        let mut p = player();
        p.playing = false;
        let key = |k: egui::Key| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        let mut press = |p: &mut Player<Sum>, k| {
            let input = egui::RawInput { events: vec![key(k)], ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| transport_keys(ui.ctx(), p));
            out.textures_delta.clear();
        };
        press(&mut p, egui::Key::ArrowRight);
        assert_eq!(p.cursor, 1);
        press(&mut p, egui::Key::ArrowLeft);
        assert_eq!(p.cursor, 0);
        press(&mut p, egui::Key::Space);
        assert!(p.playing);
    }
}
```

- [ ] **Step 2: Run them and confirm they fail.** Add `pub mod replay;` to `src/formulas/mod.rs` (next to the other `pub mod` lines), then run `cargo test -q replay::`. Expected: compile errors, because `Replay`, `Player` and `transport_keys` don't exist yet.

- [ ] **Step 3: Implement** above the tests module:

```rust
/// A state rebuilt by applying recorded operations in order.
pub trait Replay: Clone {
    type Op: Copy;
    fn apply(&mut self, op: Self::Op);
}

/// Plays a recorded log of operations against a state.
pub struct Player<S: Replay> {
    /// The state after `cursor` operations.
    pub state: S,
    pub ops: Vec<S::Op>,
    /// Number of ops applied to `state`.
    pub cursor: usize,
    pub playing: bool,
    /// Steps per second.
    pub speed: f32,
    initial: S,
    /// Fractional steps carried between frames.
    carry: f32,
}

impl<S: Replay> Player<S> {
    pub fn new(initial: S, ops: Vec<S::Op>, speed: f32) -> Self {
        Self {
            state: initial.clone(),
            ops,
            cursor: 0,
            playing: true,
            speed,
            initial,
            carry: 0.0,
        }
    }

    /// Start over with a new recording, keeping the speed.
    pub fn reset(&mut self, initial: S, ops: Vec<S::Op>) {
        *self = Self::new(initial, ops, self.speed);
    }

    /// Move to step `k`, replaying from the start when going backwards.
    pub fn seek(&mut self, k: usize) {
        let k = k.min(self.ops.len());
        if k < self.cursor {
            self.state = self.initial.clone();
            self.cursor = 0;
        }
        while self.cursor < k {
            self.state.apply(self.ops[self.cursor]);
            self.cursor += 1;
        }
    }

    /// Play forward by `dt` seconds. Returns how many steps were applied.
    pub fn advance(&mut self, dt: f32) -> usize {
        if !self.playing || self.done() {
            self.playing = false;
            return 0;
        }
        self.carry += self.speed * dt;
        let steps = (self.carry.floor() as usize).min(self.ops.len() - self.cursor);
        self.carry -= self.carry.floor();
        self.seek(self.cursor + steps);
        if self.done() {
            self.playing = false;
        }
        steps
    }

    pub fn done(&self) -> bool {
        self.cursor >= self.ops.len()
    }

    /// Play or pause; when finished, replay from the start.
    pub fn toggle(&mut self) {
        if self.done() {
            self.seek(0);
            self.playing = true;
        } else {
            self.playing = !self.playing;
        }
    }
}

/// Speed and step sliders, then Restart, Step back, Play/Pause, Step, Finish.
pub fn transport(ui: &mut egui::Ui, p: &mut Player<impl Replay>, max_speed: f32) {
    egui::Grid::new(ui.id().with("transport"))
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            ui.label("Speed");
            ui.add(
                egui::Slider::new(&mut p.speed, 1.0..=max_speed)
                    .logarithmic(true)
                    .suffix(" steps/s")
                    .custom_formatter(|v, _| format!("{v:.0}")),
            );
            ui.end_row();
            ui.label("Step");
            let mut k = p.cursor;
            if ui.add(egui::Slider::new(&mut k, 0..=p.ops.len())).changed() {
                p.playing = false;
                p.seek(k);
            }
            ui.end_row();
        });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        if ui.button("Restart").clicked() {
            p.seek(0);
            p.playing = true;
        }
        if ui.add_enabled(p.cursor > 0, egui::Button::new("Step back")).clicked() {
            p.playing = false;
            p.seek(p.cursor - 1);
        }
        let label = if p.playing {
            "Pause"
        } else if p.done() {
            "Replay"
        } else {
            "Play"
        };
        if ui.button(label).clicked() {
            p.toggle();
        }
        if ui.add_enabled(!p.done(), egui::Button::new("Step")).clicked() {
            p.playing = false;
            p.seek(p.cursor + 1);
        }
        if ui.add_enabled(!p.done(), egui::Button::new("Finish")).clicked() {
            p.playing = false;
            p.seek(p.ops.len());
        }
    });
}

/// Space plays or pauses, Left and Right step. Keys are consumed only when used.
pub fn transport_keys<S: Replay>(ctx: &egui::Context, p: &mut Player<S>) {
    let none = egui::Modifiers::NONE;
    if ctx.input_mut(|i| i.consume_key(none, egui::Key::Space)) {
        p.toggle();
    }
    if p.cursor > 0 && ctx.input_mut(|i| i.consume_key(none, egui::Key::ArrowLeft)) {
        p.playing = false;
        p.seek(p.cursor - 1);
    }
    if !p.done() && ctx.input_mut(|i| i.consume_key(none, egui::Key::ArrowRight)) {
        p.playing = false;
        p.seek(p.cursor + 1);
    }
}
```

- [ ] **Step 4: Run `cargo test -q replay::`.** Expected: 7 passed.

- [ ] **Step 5: Port sorting onto the player. This must not change behavior.**
  - In `algorithms.rs`, add `impl crate::formulas::replay::Replay for State { type Op = Op; fn apply(&mut self, op: Op) { State::apply(self, op) } }`.
  - In `SortVisualizer`, replace the fields `ops`, `state`, `cursor`, `playing`, `speed` and `carry` with one field `player: Player<State>`. Keep `initial: Vec<u32>`.
  - `sync()`: build with `self.player.reset(State::new(&self.initial), self.alg.record(&self.initial).ops)`, then set `self.player.speed = (self.player.ops.len() as f32 / 15.0).clamp(4.0, 3000.0)`.
  - `new()`: create with `Player::new(State::default(), Vec::new(), 30.0)` before calling `sync()`.
  - Keep `fn seek(&mut self, k)` and `fn done(&self)` as one-line delegations to `self.player`. Race and the tests call them.
  - Top of `paint`:

```rust
self.sync();
self.tone_cooldown -= dt;
let was_done = self.done();
let steps = self.player.advance(dt);
if steps > 0
    && let Some(v) = self.player.state.last.and_then(|op| self.sounded_value(op))
{
    self.sound(v, (2.0 / self.player.speed).clamp(0.05, 0.18));
}
if !was_done && self.done() && !self.initial.is_empty() {
    self.sweep = Some(0.0);
}
```

  - Every other `self.state` / `self.cursor` / `self.ops` / `self.playing` / `self.speed` becomes `self.player.state` / `.cursor` / `.ops` / `.playing` / `.speed`.
  - `handle_keys` becomes `replay::transport_keys(ctx, &mut self.player);`.
  - In `controls`, replace the "The player" grid and the button row with `replay::transport(ui, &mut self.player, 5000.0);`. Keep the legend and keys hint after it.
  - In `race.rs`, change `view.playing = false` to `view.player.playing = false`, `&view.ops` / `v.ops` to `v.player.ops`, `v.cursor` to `v.player.cursor`, and `v.state` to `v.player.state`.

- [ ] **Step 6: Run the gate script.** Expected: `GATE OK`. The existing `seeking_backwards_matches_forwards`, `viewports_share_data_and_resync` and `every_algorithm_sorts_and_replays_to_the_same_result` must pass without edits, except that `v.state.a` / `v.cursor` in `seeking_backwards_matches_forwards` become `v.player.state.a` / `v.player.cursor`.

- [ ] **Step 7: Commit:** `git add -A src && git commit -m "Extract a generic replay Player from the sorting visualizer"`, with the co-author line.

---

### Task 2: Shared grid with walls, mud, start and goal, and a maze generator

**Files:**
- Create: `src/formulas/graphs/mod.rs`, `src/formulas/graphs/grid.rs`
- Modify: `src/formulas/mod.rs`: `pub mod graphs;`, and add `pub graphs: graphs::Session` to `ToolEnv` with `graphs: graphs::Session::new()` in `ToolEnv::new()`.

**Interfaces:**
- Produces in `graphs/grid.rs`:
  - `pub const MUD: u8 = 5;` and `pub const GRID_W: usize = 31;` and `pub const GRID_H: usize = 19;`
  - `#[derive(Clone, Debug, PartialEq)] pub struct Grid { pub w: usize, pub h: usize, walls: Vec<bool>, weight: Vec<u8>, start: usize, goal: usize, generation: u64 }`
  - Methods:
    - `new(w, h)`: all cells open, weight 1, start at `(1, h/2)`, goal at `(w-2, h/2)`. It requires `w >= 4` and `h >= 1` (`assert!`), so the markers never share a cell.
    - Accessors: `index(x, y) -> usize`, `xy(i) -> (usize, usize)`, `len() -> usize`, `is_wall(i) -> bool`, `weight(i) -> u8`, `start() -> usize`, `goal() -> usize`, `generation() -> u64`.
    - `set_wall(i, wall: bool)`: ignored on start or goal; walling resets the weight to 1.
    - `set_weight(i, w: u8)`: ignored on walls.
    - `move_start(i) -> bool` and `move_goal(i) -> bool`: false (and nothing changes) if `i` is out of range, a wall, or the other marker.
    - `clear()`, `maze(seed: u64)`, `scatter(seed: u64, density: f32)`.
    - `neighbors(i) -> Vec<usize>`: open 4-neighbors in the fixed order up, right, down, left.
  - Every method that changes anything increments `generation`.
- Produces in `graphs/mod.rs`: `#[derive(Clone)] pub struct Session { pub grid: Rc<RefCell<grid::Grid>> }` with `Session::new()` building `Grid::new(GRID_W, GRID_H)`. Task 8 adds a `graph` field.

- [ ] **Step 1: Write the failing tests** at the bottom of `grid.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn reachable(g: &Grid) -> Vec<bool> {
        let mut seen = vec![false; g.len()];
        let mut stack = vec![g.start()];
        seen[g.start()] = true;
        while let Some(i) = stack.pop() {
            for n in g.neighbors(i) {
                if !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        seen
    }

    #[test]
    fn new_grid_is_open_with_markers_on_the_middle_row() {
        let g = Grid::new(GRID_W, GRID_H);
        assert_eq!(g.xy(g.start()), (1, GRID_H / 2));
        assert_eq!(g.xy(g.goal()), (GRID_W - 2, GRID_H / 2));
        assert!((0..g.len()).all(|i| !g.is_wall(i) && g.weight(i) == 1));
    }

    #[test]
    fn neighbors_are_open_cells_up_right_down_left() {
        let mut g = Grid::new(5, 5);
        let c = g.index(2, 2);
        assert_eq!(g.neighbors(c), vec![g.index(2, 1), g.index(3, 2), g.index(2, 3), g.index(1, 2)]);
        g.set_wall(g.index(2, 3), true);
        assert_eq!(g.neighbors(c).len(), 3);
        assert_eq!(g.neighbors(g.index(0, 0)), vec![g.index(1, 0), g.index(0, 1)]);
    }

    #[test]
    fn markers_never_go_on_walls_or_onto_each_other() {
        let mut g = Grid::new(GRID_W, GRID_H);
        let (s, t) = (g.start(), g.goal());
        let before = g.generation();
        g.set_wall(s, true);
        g.set_wall(t, true);
        assert!(!g.is_wall(s) && !g.is_wall(t));
        assert_eq!(g.generation(), before, "ignored edits do not invalidate recordings");
        let w = g.index(5, 5);
        g.set_wall(w, true);
        assert!(!g.move_start(w), "onto a wall");
        assert!(!g.move_start(t), "onto the goal");
        assert!(!g.move_goal(g.len()), "off the grid");
        assert_eq!((g.start(), g.goal()), (s, t));
        assert!(g.move_start(g.index(3, 3)));
        assert!(g.generation() > before);
    }

    #[test]
    fn mud_only_on_open_cells_and_walls_clear_it() {
        let mut g = Grid::new(GRID_W, GRID_H);
        let i = g.index(4, 4);
        g.set_weight(i, MUD);
        assert_eq!(g.weight(i), MUD);
        g.set_wall(i, true);
        assert_eq!(g.weight(i), 1);
        g.set_weight(i, MUD);
        assert_eq!(g.weight(i), 1, "walls take no mud");
    }

    #[test]
    fn mazes_connect_every_open_cell_and_depend_on_the_seed() {
        let mut a = Grid::new(GRID_W, GRID_H);
        a.maze(1);
        assert!(!a.is_wall(a.start()) && !a.is_wall(a.goal()));
        let seen = reachable(&a);
        assert!((0..a.len()).all(|i| a.is_wall(i) || seen[i]), "a perfect maze is connected");
        let mut b = Grid::new(GRID_W, GRID_H);
        b.maze(2);
        assert_ne!(a, b);
        let mut c = Grid::new(GRID_W, GRID_H);
        c.maze(1);
        assert_eq!(a, c, "same seed, same maze");
    }

    #[test]
    fn scatter_keeps_the_markers_open() {
        let mut g = Grid::new(GRID_W, GRID_H);
        g.scatter(9, 0.9);
        assert!(!g.is_wall(g.start()) && !g.is_wall(g.goal()));
        let walls = (0..g.len()).filter(|&i| g.is_wall(i)).count();
        assert!(walls > g.len() / 2);
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::grid`.** Expected: compile errors (nothing exists yet).

- [ ] **Step 3: Implement `grid.rs`.** Use `crate::rng::Rng`.
  - **`maze(seed)`** is a recursive backtracker on the odd-coordinate cells:
    1. Set every cell to a wall with weight 1.
    2. With an explicit stack (no recursion), start at `(1, 1)` and open it.
    3. Pick an unvisited odd neighbor two steps away in a random order (`rng.index`). Open it and the cell between, and push it. Pop when there are none left.
    4. Finally move `start` to `(1, h/2 | 1)` and `goal` to `(w-2, h/2 | 1)`, clamped to odd rows inside the grid, and force both open.

    With `GRID_W = 31` and `GRID_H = 19` all of these are odd.
  - **`scatter(seed, density)`** makes each non-marker cell a wall with probability `density`, and resets weights to 1.
  - Write `index`, `xy` and the bounds checks out plainly. Bump `generation` exactly once in each mutating method that changes state.
  - **`graphs/mod.rs`:**

```rust
//! Graph and tree algorithms recorded once and replayed step by step.

pub mod grid;

use std::cell::RefCell;
use std::rc::Rc;

/// What viewports of the graph tools on one board share, so algorithms can
/// be compared on exactly the same input.
#[derive(Clone)]
pub struct Session {
    pub grid: Rc<RefCell<grid::Grid>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            grid: Rc::new(RefCell::new(grid::Grid::new(grid::GRID_W, grid::GRID_H))),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run `cargo test -q graphs::grid`.** Expected: 6 passed. Then run the gate script; expected `GATE OK`.

- [ ] **Step 5: Commit** "Add the shared pathfinding grid with mazes and mud".

---

### Task 3: Recorded BFS, DFS, Dijkstra and A*

**Files:**
- Create: `src/formulas/graphs/path.rs`. Modify `graphs/mod.rs` to add `pub mod path;`.

**Interfaces:**
- Consumes: `Grid` (Task 2) and `replay::Replay` (Task 1).
- Produces:
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Search { Bfs, Dfs, Dijkstra, AStar }` with `pub const ALL: [Search; 4]` in that order.
    - `id()`: `"bfs" | "dfs" | "dijkstra" | "a-star"`
    - `name()`: `"Breadth-first search" | "Depth-first search" | "Dijkstra's algorithm" | "A* search"`
    - `weighted() -> bool`: true for Dijkstra and A*.
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum PathOp { Frontier { cell: usize, dist: u32 }, Visit(usize), Path(usize), NoPath }`
  - `pub fn record(search: Search, grid: &Grid) -> Vec<PathOp>`. The `Path` ops run start → goal in order.
  - `pub fn path_cost(grid: &Grid, search: Search, path: &[usize]) -> u32`: the number of moves for BFS/DFS; the sum of `grid.weight(cell)` over every cell entered after the start for Dijkstra/A*.
  - `#[derive(Clone, Debug, PartialEq)] pub struct PathState { pub dist: Vec<Option<u32>>, pub frontier: Vec<bool>, pub visited: Vec<bool>, pub path: Vec<usize>, pub no_path: bool, pub visits: usize, pub last: Option<PathOp> }` with `PathState::new(cells: usize)` and `impl Replay for PathState { type Op = PathOp; … }`. Applying:
    - `Frontier` sets `frontier[cell] = true` and `dist[cell] = Some(dist)`.
    - `Visit` sets `frontier = false`, `visited = true`, and `visits += 1`.
    - `Path` pushes the cell; `NoPath` sets `no_path`.
    - `last = Some(op)` always.

- [ ] **Step 1: Write the failing tests** in `path.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::graphs::grid::{Grid, MUD};
    use crate::formulas::replay::Replay;
    use crate::rng::Rng;

    fn path_of(ops: &[PathOp]) -> Vec<usize> {
        ops.iter()
            .filter_map(|op| match op {
                PathOp::Path(c) => Some(*c),
                _ => None,
            })
            .collect()
    }

    fn valid(g: &Grid, path: &[usize]) -> bool {
        path.first() == Some(&g.start())
            && path.last() == Some(&g.goal())
            && path.iter().all(|&c| !g.is_wall(c))
            && path.windows(2).all(|w| g.neighbors(w[0]).contains(&w[1]))
    }

    #[test]
    fn bfs_finds_a_manhattan_shortest_path_on_an_open_grid() {
        let g = Grid::new(11, 7);
        let path = path_of(&record(Search::Bfs, &g));
        let ((sx, sy), (tx, ty)) = (g.xy(g.start()), g.xy(g.goal()));
        assert!(valid(&g, &path));
        assert_eq!(path.len() - 1, sx.abs_diff(tx) + sy.abs_diff(ty));
    }

    #[test]
    fn a_walled_off_goal_ends_in_no_path() {
        let mut g = Grid::new(11, 7);
        for n in g.neighbors(g.goal()) {
            g.set_wall(n, true);
        }
        for s in Search::ALL {
            let ops = record(s, &g);
            assert_eq!(ops.last(), Some(&PathOp::NoPath), "{s:?}");
            assert!(path_of(&ops).is_empty());
        }
    }

    /// Review Focus 1: the markers can never share a cell, so the shortest
    /// possible search is between adjacent markers.
    #[test]
    fn adjacent_markers_give_a_two_cell_path() {
        let g = Grid::new(4, 3);
        assert_eq!((g.xy(g.start()), g.xy(g.goal())), ((1, 1), (2, 1)));
        for s in Search::ALL {
            assert_eq!(path_of(&record(s, &g)), vec![g.start(), g.goal()], "{s:?}");
        }
    }

    #[test]
    fn dfs_finds_a_valid_path_through_a_maze() {
        let mut g = Grid::new(31, 19);
        g.maze(4);
        let path = path_of(&record(Search::Dfs, &g));
        assert!(valid(&g, &path));
    }

    #[test]
    fn dijkstra_and_a_star_agree_on_the_cost_of_weighted_grids() {
        for seed in 0..100 {
            let mut g = Grid::new(21, 13);
            g.scatter(seed, 0.25);
            let mut rng = Rng::new(seed + 1000);
            for i in 0..g.len() {
                if rng.f32() < 0.2 {
                    g.set_weight(i, MUD);
                }
            }
            let d = path_of(&record(Search::Dijkstra, &g));
            let a = path_of(&record(Search::AStar, &g));
            assert_eq!(d.is_empty(), a.is_empty(), "seed {seed}");
            if !d.is_empty() {
                assert!(valid(&g, &d) && valid(&g, &a), "seed {seed}");
                assert_eq!(
                    path_cost(&g, Search::Dijkstra, &d),
                    path_cost(&g, Search::AStar, &a),
                    "seed {seed}"
                );
            }
        }
    }

    #[test]
    fn a_star_visits_no_more_cells_than_dijkstra_on_an_open_grid() {
        let g = Grid::new(31, 19);
        let visits = |s| record(s, &g).iter().filter(|op| matches!(op, PathOp::Visit(_))).count();
        assert!(visits(Search::AStar) < visits(Search::Dijkstra));
    }

    #[test]
    fn replaying_the_log_rebuilds_the_final_state() {
        let mut g = Grid::new(31, 19);
        g.maze(8);
        let ops = record(Search::Bfs, &g);
        let mut st = PathState::new(g.len());
        for &op in &ops {
            st.apply(op);
        }
        assert_eq!(st.path, path_of(&ops));
        assert!(st.visited[g.start()] && !st.no_path);
        assert_eq!(st.dist[g.goal()], Some((st.path.len() - 1) as u32));
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::path`.** Expected: compile errors.

- [ ] **Step 3: Implement `record`.** Use one `parent: Vec<Option<usize>>` and one `dist: Vec<u32>` (u32::MAX = unseen) for all four searches. Always push `Visit` when a cell is finalized; when the goal is finalized, stop and push the path.
  - **BFS:** `VecDeque`. Push `Frontier { cell: start, dist: 0 }`. When a neighbor is first seen, set its parent and dist, then push `Frontier`. Pop front, then `Visit`.
  - **DFS:** stack of `(cell, from)`. On pop, skip visited cells; otherwise set the parent to `from`, `Visit`, and push its neighbors in reverse order, so "up" is explored first, with `Frontier { dist: dist[cell] + 1 }` for each one not yet visited. Record the dist at push time.
  - **Dijkstra:** `BinaryHeap<Reverse<(u32, usize)>>`. When `dist[c] + weight(n)` beats `dist[n]`, update it, set the parent, push, and record `Frontier`. Skip stale pops (`d > dist[c]`), then `Visit`.
  - **A\*:** same as Dijkstra, but the heap key is `Reverse((d + h(n), h(n), n))` with `h = manhattan(n, goal)`. That's admissible, because every weight is ≥ 1. Ties go to the lower `h`, then the lower index, so runs are deterministic.
  - **Unweighted searches** (BFS/DFS) count moves: dist + 1. The weighted ones add `grid.weight(n)`.
  - **Path:** walk `parent` back from the goal, reverse it, and push `Path(c)` for each cell. If the goal is never finalized, push `NoPath`.
  - **`path_cost`** follows the `Interfaces` definition.

- [ ] **Step 4: Run `cargo test -q graphs::path`.** Expected: 7 passed. Then run the gate script.

- [ ] **Step 5: Commit** "Record BFS, DFS, Dijkstra and A* as replayable operation logs".

---

### Task 4: Pathfinding viewport and the BFS tool

**Files:**
- Create: `src/formulas/graphs/pathfinding.rs`, `assets/content/bfs/{doc.md,main.py,main.ts,main.rs}`
- Modify: `src/formulas/graphs/mod.rs` (`pub mod pathfinding;`)
- Modify: `src/formulas/mod.rs`:
  - `pub const GRAPHS: &str = "Graphs & Trees";`
  - a constructor `fn path<const S: usize>(env: &ToolEnv) -> Box<dyn Formula> { Box::new(graphs::pathfinding::Pathfinding::new(graphs::path::Search::ALL[S], env.graphs.clone())) }`
  - a `tool!("bfs", "Breadth-first search", GRAPHS, "BFS", path::<0>)` entry appended after the last `SORTING` entry.
- Modify: `src/cad/chrome.rs` `short()`: add `formulas::GRAPHS => "Graphs",`.
- Modify: `src/cad/icons.rs`: a `"bfs"` icon arm. Draw a 3×3 dot grid with concentric rings spreading from the left-middle dot. Use only `p.dot`, `p.line`, `p.closed`, `p.curve`, matching the neighboring arms.
- Modify: `README.md`: a "Graphs & Trees" row in the tool table, before "Sorting".

**Interfaces:**
- Consumes: `Grid`, `Session`, `Search`, `record`, `PathOp`, `PathState`, `path_cost` (Tasks 2 and 3), and `Player`, `transport`, `transport_keys` (Task 1).
- Produces:
  - `pub struct Pathfinding`, with `Pathfinding::new(search: Search, session: Session) -> Self`.
  - `pub fn cell_at(rect: egui::Rect, w: usize, h: usize, p: egui::Pos2) -> Option<usize>`
  - `pub fn board_rect(rect: egui::Rect, w: usize, h: usize) -> egui::Rect`. This is the largest square-celled rect centered in `rect`, below a 30-point header strip.

**Behavior spec for `Pathfinding`** (implements `Formula`):
- **Fields:** `search`, `session`, `generation: u64` (the grid generation last recorded; start at `u64::MAX`), `player: Player<PathState>`, `mud: bool` (brush mode), and `stroke: Option<Stroke>`. `enum Stroke { Walls(bool) /* painting walls on or off */, MoveStart, MoveGoal, Mud(bool) }`.
- **`sync()`:** if `grid.generation() != self.generation`, re-record with `self.player.reset(PathState::new(grid.len()), record(self.search, &grid))` and store the generation. Call it at the top of `paint` and after `controls`.
- **`id`/`name`:** from `search.id()` / `search.name()`. Category `super::GRAPHS`.
- **`summary`, one per search:**
  - BFS: "explores in rings of equal distance, so the first path it finds is the shortest in steps".
  - DFS: "dives down one corridor before backing up; finds a path, rarely the shortest".
  - Dijkstra: "settles cells in order of total cost, so mud (weight 5) is avoided when a detour is cheaper".
  - A*: "is Dijkstra steered by a straight-line guess of the remaining distance, so it visits fewer cells".
- **`notes`:** real uses: maze solving, game AI and navigation meshes, GPS routing, network routing, flood fill.
- **`equations`, one set per search:**
  - BFS: `dist(n) = dist(u) + 1` and "visit cells in order of dist".
  - DFS: "follow the newest frontier cell first (a stack)".
  - Dijkstra: `dist(n) = min(dist(n), dist(u) + w(n))`.
  - A*: `f(n) = g(n) + h(n)` and `h(n) = |x − x`<sub>goal</sub>`| + |y − y`<sub>goal</sub>`|`, built with `MathText` `.sub("goal")`.
- **`params()`:** `vec![("visited", state.visits as f64), ("path", path_len_or_0)]`.
- **`stage_height`:** `width * GRID_H as f32 / GRID_W as f32 + 64.0`. **`min_size`:** `vec2(420.0, stage_height(420))`.
- **`stage_sense`:** `Sense::click_and_drag()`. **`is_animating`:** `self.player.playing`.
- **`interact`** (cell from `cell_at(rect, …)`):
  - On drag start or click, choose the stroke:
    - on the start cell → `MoveStart`; on the goal → `MoveGoal`;
    - else if `self.mud` → `Mud(!weight_is_mud)`;
    - else `Walls(!is_wall)`.
  - While dragging, apply the stroke to the cell under the pointer: `set_wall` / `set_weight(MUD or 1)` / `move_start` / `move_goal`.
  - Clear the stroke on drag stop. A single click applies the stroke once.
  - Every change goes through `self.session.grid.borrow_mut()`.
- **`paint`:**
  - `self.sync(); self.player.advance(dt);`
  - Header strip: `"{name}  ·  visited {visits}  ·  path {len} steps"`. Weighted searches show `cost {path_cost}`, and the text becomes `"No path: the goal is walled off"` when `state.no_path`.
  - Cells, in this order of precedence: wall `palette.ink`; mud `palette.y.gamma_multiply(0.35)`; path `palette.trace`; frontier `palette.x.gamma_multiply(0.55)`; visited `palette.fill`; open `palette.paper`.
  - Grid lines `palette.rule`. Start: filled circle in `palette.x`. Goal: ring in `palette.y`.
  - When the board's cell size is ≥ 18 points and the search is weighted, draw `dist` numbers in visited cells (font 9).
  - The cell of the latest op (`state.last`) gets a 2-point `palette.ink` outline.
- **`controls`:**
  - Presets: `Empty` (`grid.clear()`), `Maze` (`grid.maze(seed)`), `Random walls` (`grid.scatter(seed, 0.28)`). Use a `seed: u64` field that increments on each click.
  - A `mud` checkbox "Paint mud (weight 5)".
  - Then `replay::transport(ui, &mut self.player, 2000.0)`, a small legend row (wall, mud, frontier, visited, path) in the cell colors, and a hint "Drag to draw walls, drag S or G to move them; Space, Left and Right step".
- **`handle_keys`:** `replay::transport_keys(ctx, &mut self.player)`.

**Content for `bfs` (`assets/content/bfs/`):**
- `doc.md`, titled `# Breadth-first search`, with the ten sections. Cover:
  - **Formula:** the queue invariant and `dist(n) = dist(u) + 1`.
  - **Intuition:** rings, shortest in edges.
  - **Algorithm:** pseudocode.
  - **Complexity:** O(V + E) time and O(V) space.
  - **Parameters in the app:** every control above.
  - **Numerical notes and pitfalls:**
    - marking visited at enqueue, not dequeue (else duplicates);
    - weighted graphs need Dijkstra;
    - recursion is not BFS.
  - **Applications:** shortest paths in unweighted graphs, social-network degrees, web crawling, flood fill, bipartiteness.
  - **History:** Konrad Zuse (1945, published 1972) and Edward F. Moore (1959, maze shortest paths); C. Y. Lee (1961, wire routing).
  - **References:** CLRS ch. 22; Sedgewick & Wayne *Algorithms* 4.1; Wikipedia.
- `main.py`, `main.ts`, `main.rs`: a 31×19 grid with the default markers. Clicking toggles walls, a key `m` makes a maze, and a key `r` restarts. BFS is recorded first and replayed at a fixed speed with the same colors. Header comment in the style of `assets/content/matrix-transform/main.*`. Pure ASCII.

- [ ] **Step 1: Write the failing tests** in `pathfinding.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::graphs::{Session, path::Search};
    use egui::{Rect, pos2, vec2};

    #[test]
    fn cells_map_to_and_from_the_board() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(620.0, 410.0));
        let board = board_rect(rect, 31, 19);
        let cell = board.width() / 31.0;
        assert!((board.height() / 19.0 - cell).abs() < 1e-3, "square cells");
        let p = board.min + vec2(cell * 2.5, cell * 3.5);
        assert_eq!(cell_at(rect, 31, 19, p), Some(3 * 31 + 2));
        assert_eq!(cell_at(rect, 31, 19, board.min - vec2(1.0, 1.0)), None);
        assert_eq!(cell_at(rect, 31, 19, board.max + vec2(1.0, 1.0)), None);
    }

    /// Review Focus 5: when the shared grid changes mid-replay, every viewport
    /// re-records and its cursor restarts inside the new log.
    #[test]
    fn viewports_share_the_grid_and_rerecord_when_it_changes() {
        let session = Session::new();
        let mut bfs = Pathfinding::new(Search::Bfs, session.clone());
        let mut dfs = Pathfinding::new(Search::Dfs, session.clone());
        bfs.player.seek(bfs.player.ops.len());
        session.grid.borrow_mut().maze(3);
        bfs.sync();
        dfs.sync();
        assert_eq!(bfs.player.cursor, 0);
        assert!(bfs.player.cursor <= bfs.player.ops.len());
        assert_eq!(bfs.generation, session.grid.borrow().generation());
        assert_eq!(dfs.generation, bfs.generation);
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::pathfinding`.** Expected: compile errors.
- [ ] **Step 3: Implement `pathfinding.rs`** to the behavior spec, plus the registration, toolbar name, icon and README row.
- [ ] **Step 4: Run `cargo test -q`.** The new tests pass. The existing every-tool tests now demand content, so write `assets/content/bfs/*` until `every_tool_has_docs_and_code`, `docs_have_every_section_in_order`, `code_is_ascii`, `every_tool_has_its_own_icon`, `all_text_has_glyphs`, `every_tool_runs_on_the_canvas`, `every_category_has_a_toolbar_name` and `showing_controls_leaves_parameters_unchanged` all pass.
- [ ] **Step 5: Run `check-export.sh bfs`.** Expected: `EXPORTS OK: bfs`. Then run the gate script; expected `GATE OK`.
- [ ] **Step 6: Commit** "Add the pathfinding viewport and the breadth-first search tool".

---

### Task 5: DFS, Dijkstra and A* tools

**Files:**
- Modify: `src/formulas/mod.rs`. Add three `tool!` entries right after `bfs`:
  - `tool!("dfs", "Depth-first search", GRAPHS, "DFS", path::<1>)`
  - `tool!("dijkstra", "Dijkstra's algorithm", GRAPHS, "DIJ", path::<2>)`
  - `tool!("a-star", "A* search", GRAPHS, "AST", path::<3>)`
- Modify: `src/cad/icons.rs`, adding three icon arms:
  - `"dfs"`: one long zigzag line diving to the bottom-right;
  - `"dijkstra"`: three dots joined by lines, with one thick shortest leg;
  - `"a-star"`: a star-shaped `p.closed` polygon with a dot target.
- Create: `assets/content/{dfs,dijkstra,a-star}/{doc.md,main.py,main.ts,main.rs}`

**Interfaces:**
- Consumes: `Pathfinding::new` and `Search::ALL` order (Tasks 3 and 4).

**Content requirements** (each `doc.md` has the ten sections; the exports follow the Task 4 export spec with that algorithm):
- **dfs:**
  - Complexity O(V + E).
  - Pitfalls: not shortest, recursion depth limits (use an explicit stack), and visited-on-pop vs push.
  - Applications: maze generation, topological sort, cycle detection, connected components, backtracking.
  - History: Charles Pierre Trémaux (19th-century maze solving); Hopcroft & Tarjan (1973).
- **dijkstra:**
  - Formula: relaxation `dist(v) = min(dist(v), dist(u) + w(u, v))`.
  - Complexity: O((V + E) log V) with a binary heap.
  - Pitfalls: negative weights break it (use Bellman–Ford), and stale heap entries must be skipped.
  - Applications: GPS routing, OSPF/IS-IS network routing, robotics.
  - History: Edsger W. Dijkstra (1956, published 1959).
- **a-star:**
  - Formula: `f = g + h`, admissible and consistent heuristics, Manhattan distance on 4-connected grids.
  - Complexity: worst case like Dijkstra; in practice far fewer expansions.
  - Pitfalls: an inadmissible `h` loses optimality, and tie-breaking affects work.
  - Applications: games, navigation meshes, puzzle solving (the 15-puzzle).
  - History: Hart, Nilsson & Raphael (1968, Shakey the robot project at SRI).

- [ ] **Step 1: Write the failing test.** Add the three `tool!` entries and icon arms. Run `cargo test -q`. Expected: `every_tool_has_docs_and_code` fails, listing the 9 missing files for dfs, dijkstra and a-star.
- [ ] **Step 2: Write the three content directories.**
- [ ] **Step 3: Run `cargo test -q`** until it passes. Then `check-export.sh dfs`, `check-export.sh dijkstra` and `check-export.sh a-star` must each print `EXPORTS OK`. Then run the gate script.
- [ ] **Step 4: Commit** "Add depth-first, Dijkstra and A* pathfinding tools".

---

### Task 6: BST/AVL tree with snapshot frames

**Files:**
- Create: `src/formulas/graphs/tree.rs`. Modify `graphs/mod.rs` to add `pub mod tree;`.

**Interfaces:**
- Produces:
  - `#[derive(Clone, Debug, PartialEq)] pub struct Node { pub key: i32, pub left: Option<usize>, pub right: Option<usize>, pub height: i32 }`. A leaf has height 1.
  - `#[derive(Clone, Debug, Default, PartialEq)] pub struct Tree { nodes: Vec<Option<Node>>, root: Option<usize> }`. This is an arena: deleted nodes become `None`, and ids are never reused, so frames can tween by id.
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Rotation { Left(usize), Right(usize) }`. The id is the node that moves down.
  - `#[derive(Clone, Debug, PartialEq)] pub struct Frame { pub tree: Tree, pub focus: Option<usize>, pub rotation: Option<Rotation>, pub caption: String }`
  - Methods:
    - `insert(&mut self, key: i32, balance: bool, frames: &mut Vec<Frame>) -> bool`
    - `delete(&mut self, key: i32, balance: bool, frames: &mut Vec<Frame>) -> bool`
    - `contains(&self, key: i32) -> bool`
    - `root() -> Option<usize>`, `node(id) -> Option<&Node>`, `len() -> usize`
    - `inorder() -> Vec<usize>`, `preorder() -> Vec<usize>`, `postorder() -> Vec<usize>`, `keys() -> Vec<i32>` (in order)
    - `balance_factor(id) -> i32` (height(left) − height(right))
    - `layout() -> Vec<(usize, f32, usize)>`: (id, x in 0..1 by in-order rank as `(rank + 0.5) / len`, depth).
  - Captions are ASCII, for example:
    - `"insert 7: 7 < 12, go left"`, `"insert 7: new leaf"`
    - `"balance factor 2 at 12: rotate right"`
    - `"delete 9: not found"`, `"insert 7: 7 is already in the tree"`
    - `"delete 12: two children, copy successor 15"`

- [ ] **Step 1: Write the failing tests** in `tree.rs`:

```rust
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
        assert!(keys.windows(2).all(|w| w[0] < w[1]), "in-order is sorted: {keys:?}");
        for id in t.inorder() {
            let n = t.node(id).unwrap();
            let h = |c: Option<usize>| c.and_then(|c| t.node(c)).map_or(0, |n| n.height);
            assert_eq!(n.height, 1 + h(n.left).max(h(n.right)), "cached height of {}", n.key);
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
        assert!(frames.last().unwrap().caption.contains("already in the tree"));
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
        let keys = |ids: Vec<usize>| ids.iter().map(|&i| t.node(i).unwrap().key).collect::<Vec<_>>();
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
    fn node_ids_survive_rotations() {
        let (t1, _) = build(&[1, 2], true);
        let id_of_2 = t1.inorder()[1];
        let (t2, _) = build(&[1, 2, 3], true);
        assert_eq!(t2.node(id_of_2).unwrap().key, 2, "same id, same key after rotating");
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::tree`.** Expected: compile errors.
- [ ] **Step 3: Implement** with recursive helpers that return the new subtree root id:
  - `fn insert_at(&mut self, at: Option<usize>, key, balance, frames, caption_prefix) -> (Option<usize>, bool)`
  - `fn delete_at(&mut self, at: Option<usize>, key, balance, frames) -> (Option<usize>, bool)`
  - `fn rebalance(&mut self, id, frames) -> usize` (LL/RR/LR/RL from the balance factors of the node and the heavy child)
  - `fn rotate_left(&mut self, id) -> usize` and `fn rotate_right(&mut self, id) -> usize`, which update heights.

  **Frames:**
  - Push one frame per comparison (focus = current node, caption "… go left/right").
  - Push one frame after the leaf is placed.
  - For each rotation, push one frame **before** it (`rotation: Some(..)`, caption "balance factor … rotate …") and one **after** it (`rotation: None`).
  - Push a final frame for deletes ("delete K: done").

  Every frame clones `self` (the tree is small). A new node gets `id = nodes.len()` and is pushed. Deleting sets `nodes[id] = None`. Two-child delete: copy the successor's key into the node (caption includes `successor {k}`), then delete the successor from the right subtree.

- [ ] **Step 4: Run `cargo test -q graphs::tree`.** Expected: 8 passed. Then run the gate script.
- [ ] **Step 5: Commit** "Add a BST/AVL tree that records a frame for every step".

---

### Task 7: Binary search tree viewport and tool

**Files:**
- Create: `src/formulas/graphs/bst.rs`, `assets/content/binary-search-tree/{doc.md,main.py,main.ts,main.rs}`
- Modify: `src/formulas/graphs/mod.rs` (`pub mod bst;`)
- Modify: `src/formulas/mod.rs`: `tool!("binary-search-tree", "Binary search tree", GRAPHS, "BST", new::<graphs::bst::SearchTree>)` after `a-star`.
- Modify: `src/cad/icons.rs` (`"binary-search-tree"`: a root dot with two children and two grandchildren joined by lines), `README.md` (append to the Graphs & Trees row).

**Interfaces:**
- Consumes: `Tree`, `Frame`, `Rotation` (Task 6), and `Player`, `Replay`, `transport`, `transport_keys` (Task 1).
- Produces:
  - `pub struct SearchTree` with `Default`.
  - `#[derive(Clone, Debug, Default, PartialEq)] pub struct FrameIndex(pub usize);` with `impl Replay for FrameIndex { type Op = (); fn apply(&mut self, _: ()) { self.0 += 1 } }`
  - `pub fn tween(a: &[(usize, f32, usize)], b: &[(usize, f32, usize)], t: f32) -> Vec<(usize, f32, f32)>`: (id, x, depth as f32) interpolated by id. Ids only in `b` take their `b` position; ids only in `a` are dropped.

**Behavior spec for `SearchTree`:**
- **Fields:**
  - `tree: Tree`, `balance: bool` (default true);
  - `history: Vec<i32>` (keys inserted, minus deleted, for rebuilding);
  - `frames: Vec<Frame>`, `player: Player<FrameIndex>` (ops = `vec![(); frames.len() - 1]`);
  - `hold: f32` (time into the current frame, for tweening);
  - `key: i32` (the value field, default 25), `seed: u64`.
- **`Default`:** insert `[50, 30, 70, 20, 40, 60, 80, 35, 45, 65]` with balancing, keep only the last frame, and leave the player done.
- **Each action** (Insert, Delete, Find, a traversal) builds a fresh `frames` vec starting with a frame of the current tree. It then calls the operation and resets the player at speed 2.5 frames/s, playing.
  - Find adds frames walking the comparisons with captions; it does not change the tree.
  - Traversal buttons "In-order", "Pre-order" and "Post-order" add one frame per visited node (focus = node; caption accumulates the visited keys, e.g. `"in-order: 20 30 35"`).
  - **Random 15:** clear, then insert 15 distinct keys from `Rng::new(seed)` in −99..=99.
  - **Clear.**
  - **The AVL checkbox** rebuilds from `history` with the new flag, showing only the final frame.
- **`paint`:**
  - Advance the player.
  - Positions: `tween(prev_frame.tree.layout(), frame.tree.layout(), smoothstep(hold * speed))`.
  - Nodes: circles of radius 15 with the key centered (font 12). Edges: `palette.rule` lines from each node to its children.
  - The focus node is filled with `palette.trace`; the rotation pivot is ringed in `palette.y`. With balancing on, show the balance factor above each node (font 9, `palette.muted`) and color it `palette.y` when |bf| ≥ 2.
  - Caption along the bottom (font 13, `palette.ink`). Header: `"n = {len}, height = {h}"`.
- **`controls`:**
  - A `DragValue` for `key` in −99..=99, then the buttons Insert / Delete / Find.
  - A row: Random 15 / Clear / the "Balance (AVL)" checkbox.
  - A row: In-order / Pre-order / Post-order.
  - Then `transport(ui, &mut self.player, 20.0)`.
- **`params()`:** `vec![("n", len), ("height", root height or 0)]`.
- **`equations`:**
  - "left subtree < key < right subtree"
  - `bf = h(left) − h(right),   |bf| ≤ 1`
  - `h = 1 + max(h(left), h(right))`
- **Category `GRAPHS`. `handle_keys`:** `transport_keys`.

**Content (`binary-search-tree`):**
- **Overview:** the BST property, and AVL as self-balancing.
- **Formula:** the balance factor and the four rotation cases.
- **Complexity:** O(h) per operation; O(log n) for AVL (h < 1.44 log₂(n + 2)) and O(n) for an unbalanced chain.
- **Pitfalls:** sorted input degenerates, duplicates, deleting with two children, recursion depth.
- **Applications:** ordered maps and sets (`std::collections::BTreeMap` is a B-tree), database indexes, interval trees, and the Linux scheduler's red-black tree.
- **History:** BST by P. F. Windley, A. D. Booth, A. J. T. Colin and T. N. Hibbard (around 1960); AVL by Georgy Adelson-Velsky and Evgenii Landis (1962); red-black trees by Guibas & Sedgewick (1978).
- **Exports:** insert a fixed list with AVL balancing and draw the tree. Clicking adds a random key, `d` deletes the last one. Each rotation is shown for half a second.

- [ ] **Step 1: Write the failing tests** in `bst.rs`:

```rust
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
        assert!(t.tree.inorder().iter().all(|&id| t.tree.balance_factor(id).abs() <= 1));
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
```

  `set_balance(&mut self, on: bool)` is the method the checkbox calls. Add it to the spec'd struct.
- [ ] **Step 2: Run `cargo test -q graphs::bst`.** Expected: compile errors.
- [ ] **Step 3: Implement** to the spec, plus the registration, icon, content and README.
- [ ] **Step 4: Run `cargo test -q`** until every every-tool test passes. Then `check-export.sh binary-search-tree` must print `EXPORTS OK`, and the gate script must print `GATE OK`.
- [ ] **Step 5: Commit** "Add the binary search tree tool with AVL rotations".

---

### Task 8: MST graph with recorded Kruskal and Prim

**Files:**
- Create: `src/formulas/graphs/mst.rs`
- Modify: `src/formulas/graphs/mod.rs`: `pub mod mst;`, and add `pub graph: Rc<RefCell<mst::Graph>>` to `Session`, built with `mst::Graph::random(14, 7)`.

**Interfaces:**
- Consumes: `crate::formulas::voronoi::cell(seeds: &[egui::Pos2], i: usize, bounds: egui::Rect, gap: f32) -> Vec<(egui::Pos2, Option<usize>)>` and `voronoi::neighbors(&poly) -> impl Iterator<Item = usize>`. These are the Delaunay neighbors (`src/formulas/voronoi.rs` lines 18 and 89).
- Produces:
  - `#[derive(Clone, Copy, Debug, PartialEq)] pub struct Edge { pub a: usize, pub b: usize, pub w: f32 }` with `a < b`, and `w` the Euclidean length × 100 (points live in the unit square).
  - `#[derive(Clone, Debug, PartialEq)] pub struct Graph { points: Vec<[f32; 2]>, edges: Vec<Edge>, generation: u64 }`
    - `random(n, seed) -> Self`: points in 0.08..0.92 with min spacing 0.08 (rejection sampling, at most 1000 tries; fewer points if exhausted).
    - Accessors: `points() -> &[[f32; 2]]`, `edges() -> &[Edge]`, `generation() -> u64`.
    - `move_point(i, p: [f32; 2])`: clamps to 0.02..0.98. If it lands within 0.01 of another point, it is nudged 0.012 to the right (wrapping left at the edge), so points never coincide. Rebuilds edges and bumps the generation.
  - `fn delaunay(points: &[[f32; 2]]) -> Vec<Edge>`: built from `voronoi::cell` over `Rect::from_min_max(pos2(0,0), pos2(1,1))` with gap 0, deduplicated with `a < b` and sorted by `(a, b)`.
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Spanning { Kruskal, Prim }` with `ALL`, `id()` (`"kruskal" | "prim"`) and `name()` (`"Kruskal's algorithm" | "Prim's algorithm"`).
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum MstOp { Consider(usize), Accept(usize), Reject(usize), Union { child: usize, parent: usize }, Reach(usize) }`. Edge indices index `graph.edges()`; `Reach` is a vertex joining Prim's tree.
  - `pub fn record(alg: Spanning, g: &Graph) -> Vec<MstOp>`
  - `pub fn accepted(ops: &[MstOp]) -> Vec<usize>`
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum EdgeState { Unseen, Considering, Accepted, Rejected }`
  - `#[derive(Clone, Debug, PartialEq)] pub struct MstState { pub edges: Vec<EdgeState>, pub parent: Vec<usize>, pub in_tree: Vec<bool>, pub weight: f32, pub accepted: usize, pub last: Option<MstOp> }` with `MstState::new(g: &Graph)` (parent[i] = i) and `impl Replay` (`Consider` marks Considering; `Accept` marks Accepted and adds its weight; `Reject` marks Rejected; `Union` sets `parent[child] = parent`; `Reach` sets `in_tree[v]`).

- [ ] **Step 1: Write the failing tests** in `mst.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn weight(g: &Graph, edges: &[usize]) -> f32 {
        edges.iter().map(|&e| g.edges()[e].w).sum()
    }

    fn spans(g: &Graph, edges: &[usize]) -> bool {
        let n = g.points().len();
        let mut root: Vec<usize> = (0..n).collect();
        fn find(r: &mut Vec<usize>, x: usize) -> usize {
            if r[x] == x { x } else { let p = r[x]; let q = find(r, p); r[x] = q; q }
        }
        for &e in edges {
            let (a, b) = (find(&mut root, g.edges()[e].a), find(&mut root, g.edges()[e].b));
            root[a] = b;
        }
        let r0 = find(&mut root, 0);
        (0..n).all(|v| find(&mut root, v) == r0)
    }

    #[test]
    fn delaunay_edges_are_finite_unique_and_connect_every_point() {
        let g = Graph::random(14, 7);
        assert!(g.points().len() >= 10);
        assert!(g.edges().iter().all(|e| e.a < e.b && e.w.is_finite() && e.w > 0.0));
        let mut pairs: Vec<_> = g.edges().iter().map(|e| (e.a, e.b)).collect();
        pairs.dedup();
        assert_eq!(pairs.len(), g.edges().len());
        let all: Vec<usize> = (0..g.edges().len()).collect();
        assert!(spans(&g, &all));
    }

    #[test]
    fn kruskal_and_prim_find_the_same_minimum_weight() {
        for seed in 0..100 {
            let g = Graph::random(12, seed);
            let k = accepted(&record(Spanning::Kruskal, &g));
            let p = accepted(&record(Spanning::Prim, &g));
            let n = g.points().len();
            assert_eq!((k.len(), p.len()), (n - 1, n - 1), "seed {seed}");
            assert!(spans(&g, &k) && spans(&g, &p), "seed {seed}");
            assert!((weight(&g, &k) - weight(&g, &p)).abs() < 1e-3, "seed {seed}");
        }
    }

    #[test]
    fn small_graphs_match_brute_force() {
        for seed in 0..30 {
            let g = Graph::random(6, seed);
            let m = g.edges().len();
            let n = g.points().len();
            let mut best = f32::INFINITY;
            for mask in 0u32..(1 << m) {
                if mask.count_ones() as usize != n - 1 { continue; }
                let set: Vec<usize> = (0..m).filter(|i| mask >> i & 1 == 1).collect();
                if spans(&g, &set) { best = best.min(weight(&g, &set)); }
            }
            let k = accepted(&record(Spanning::Kruskal, &g));
            assert!((weight(&g, &k) - best).abs() < 1e-3, "seed {seed}");
        }
    }

    /// Review Focus 4: dragging a point onto another, or lining points up,
    /// keeps the triangulation finite and the MST spanning.
    #[test]
    fn coincident_and_collinear_points_still_give_a_spanning_tree() {
        let mut g = Graph::random(8, 3);
        let target = g.points()[1];
        g.move_point(0, target);
        assert!(g.points()[0] != g.points()[1]);
        for i in 0..g.points().len() {
            g.move_point(i, [0.1 + 0.1 * i as f32, 0.5]);
        }
        let k = accepted(&record(Spanning::Kruskal, &g));
        assert_eq!(k.len(), g.points().len() - 1);
        assert!(spans(&g, &k));
        assert!(g.edges().iter().all(|e| e.w.is_finite()));
    }

    #[test]
    fn replay_accumulates_the_tree_weight() {
        let g = Graph::random(12, 5);
        let ops = record(Spanning::Prim, &g);
        let mut st = MstState::new(&g);
        for &op in &ops {
            crate::formulas::replay::Replay::apply(&mut st, op);
        }
        let k = accepted(&ops);
        assert_eq!(st.accepted, k.len());
        assert!((st.weight - weight(&g, &k)).abs() < 1e-3);
        assert!(st.in_tree.iter().all(|&t| t));
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::mst`.** Expected: compile errors.
- [ ] **Step 3: Implement.**
  - **Kruskal:**
    1. Edge indices sorted by `(w, index)` (`total_cmp`).
    2. Union-find by size, *without* path compression, so `parent` stays a readable forest.
    3. For each edge, `Consider(e)`. If the roots differ: `Accept(e)` and `Union { child: smaller root, parent: larger root }`. Otherwise `Reject(e)`.
    4. Stop at n − 1 accepted edges.
  - **Prim:**
    1. `Reach(0)`, then push 0's edges onto a `BinaryHeap<Reverse<(OrderedW, e)>>`. Use a tiny `#[derive(PartialEq, PartialOrd)]` wrapper, or `(u32 bits of w as ordered, e)`: `w` is positive, so `w.to_bits()` orders correctly.
    2. Pop, `Consider(e)`. If both ends are in the tree, `Reject(e)`. Otherwise `Accept(e)`, `Reach(v)`, and push v's edges whose other end is outside the tree.
  - **Collinear points:** if they make `voronoi::cell` produce no neighbor pairs for some point, the `spans` test fails. In that case, add the edges between consecutive points sorted by (x, y) before deduplication. This keeps the graph connected.

- [ ] **Step 4: Run `cargo test -q graphs::mst`.** Expected: 5 passed. Then run the gate script.
- [ ] **Step 5: Commit** "Record Kruskal's and Prim's minimum spanning trees on a Delaunay graph".

---

### Task 9: MST viewport, Kruskal and Prim tools, docs

**Files:**
- Create: `src/formulas/graphs/spanning.rs`, `assets/content/{kruskal,prim}/{doc.md,main.py,main.ts,main.rs}`
- Modify: `src/formulas/graphs/mod.rs` (`pub mod spanning;`)
- Modify: `src/formulas/mod.rs`:
  - a constructor `fn mst<const S: usize>(env: &ToolEnv) -> Box<dyn Formula> { Box::new(graphs::spanning::SpanningTree::new(graphs::mst::Spanning::ALL[S], env.graphs.clone())) }`
  - `tool!("kruskal", "Kruskal's algorithm", GRAPHS, "KRU", mst::<0>)` and `tool!("prim", "Prim's algorithm", GRAPHS, "PRI", mst::<1>)` after `binary-search-tree`.
- Modify: `src/cad/icons.rs` (`"kruskal"`: four dots with three thick tree edges and one thin rejected edge; `"prim"`: a dot with edges growing out to three dots), `README.md` (complete the Graphs & Trees row).
- Modify: `CLAUDE.md`. Under Architecture, add a short paragraph: "**Step-through algorithms.** `formulas/replay.rs` (`Replay`, `Player`, `transport`, `transport_keys`) plays any recorded op log; sorting, pathfinding and MST use op logs, the BST replays snapshot frames through `FrameIndex`. `ToolEnv.graphs` shares one grid and one graph per board."

**Interfaces:**
- Consumes: `Graph`, `Spanning`, `record`, `MstOp`, `MstState`, `EdgeState` (Task 8), `Session.graph`, and `Player`/`transport`/`transport_keys`.
- Produces: `pub struct SpanningTree`, with `SpanningTree::new(alg: Spanning, session: Session) -> Self`.

**Behavior spec:**
- **`sync()` on graph generation**, the same as Pathfinding.
- **Points** are drawn in the unit square mapped to the stage below a 30-point header, with a 20-point margin.
- **Edges by state:** Unseen `palette.rule`; Considering `palette.y`, width 3; Accepted `palette.trace`, width 3; Rejected `palette.muted.gamma_multiply(0.4)`, dashed.
- **Kruskal** colors vertices by their union-find root (the root's index into a 6-color cycle of `palette.x`, `.y`, `.trace`, `.ink`, `.muted`, `palette.x.gamma_multiply(0.6)`). It shows a side list of the next 10 edges in sorted order with their weights. Use the narrow right column when the stage is ≥ 560 wide, otherwise hide it.
- **Prim** fills in-tree vertices with `palette.trace`, others with `palette.card` with an ink ring.
- **Header:** `"{name}  ·  tree weight {w:.0}  ·  edges {accepted} / {n-1}"`.
- **Dragging a vertex** (`draw::pick` radius 14) moves it via `graph.borrow_mut().move_point`.
- **Controls:**
  - "New points" (`Graph::random(n, seed)`, with a slider for `n` in 6..=24 and **no** `step_by`);
  - then `transport(ui, &mut self.player, 60.0)`;
  - a legend;
  - the hint "Drag points to move them".
- **`params()`:** `vec![("weight", w), ("edges", accepted)]`.
- **`equations`:**
  - Kruskal: "sort edges by weight; add each one that joins two different trees".
  - Prim: "grow one tree; add the lightest edge leaving it".
  - Both: `minimize Σ w(e) over the n − 1 tree edges`.

**Content:**
- **kruskal:**
  - Complexity: O(E log E).
  - Pitfalls: union-find without union by rank degenerates; ties.
  - Applications: network design, clustering (single-linkage = cutting the MST's longest edges), image segmentation, approximating TSP.
  - History: Joseph Kruskal (1956); Otakar Borůvka (1926) first solved the problem for Moravian electrical networks.
- **prim:**
  - Complexity: O(E log V) with a binary heap, O(V²) with an array for dense graphs.
  - Pitfalls: needs a connected graph, and stale heap entries.
  - Applications: cable and pipe layout, and maze generation (randomized Prim).
  - History: Vojtěch Jarník (1930), Robert Prim (1957), Dijkstra (1959).
- **Exports:** 14 random points, Delaunay edges or, simpler, all pairs within distance 0.35, then animate the algorithm. Clicking regenerates.

- [ ] **Step 1: Write the failing tests** in `spanning.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::graphs::{Session, mst::Spanning};

    /// Review Focus 5 for graphs: moving a point mid-replay re-records every
    /// viewport sharing the graph.
    #[test]
    fn moving_a_point_rerecords_every_viewport() {
        let session = Session::new();
        let mut k = SpanningTree::new(Spanning::Kruskal, session.clone());
        let mut p = SpanningTree::new(Spanning::Prim, session.clone());
        k.player.seek(3);
        session.graph.borrow_mut().move_point(0, [0.5, 0.5]);
        k.sync();
        p.sync();
        assert_eq!(k.player.cursor, 0);
        assert_eq!(k.generation, session.graph.borrow().generation());
        assert_eq!(p.generation, k.generation);
    }
}
```

- [ ] **Step 2: Run `cargo test -q graphs::spanning`.** Expected: compile errors.
- [ ] **Step 3: Implement** the viewport, registration, icons, content, README and the CLAUDE.md paragraph.
- [ ] **Step 4: Run `cargo test -q`** until every every-tool test passes. Then `check-export.sh kruskal` and `check-export.sh prim` must each print `EXPORTS OK`, the gate script must print `GATE OK`, and `cargo check -q --lib --target wasm32-unknown-unknown` must exit 0.
- [ ] **Step 5: Commit** "Add Kruskal's and Prim's minimum spanning tree tools".

---

## Final verification (controller, after all tasks)

1. Run the gate script, and `cargo check --lib --target wasm32-unknown-unknown`.
2. Build the web version and drive it in headless Chromium (as for PR #8):
   - `#bfs`: draw walls, then the Maze preset; the path appears.
   - `#a-star`: visits fewer cells than `#dijkstra` on the same maze.
   - Wall off the goal: the "No path" caption appears.
   - `#binary-search-tree`: Insert 1, 2, 3 in a cleared tree; the rotation animates.
   - `#kruskal` and `#prim`: the same tree weight in the header.
   - Light and dark themes.
3. Run a whole-branch code review, then push and open the PR.
