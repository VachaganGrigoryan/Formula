# Scope 6: Sheet files (`.fwb`)

> Built inline in the main thread (user preference: no background subagents). The first step copies
> this plan to `docs/superpowers/plans/2026-10-18-sheet-files.md` on the branch. Steps use `- [ ]`.

## Context

Scopes 0, 5, 4, 2, 3 and 1 are merged (main `3e778af`, PR #13). Scope 6 is the last one in the
approved order. Today a board survives only in the browser or eframe storage. Each tool's own
settings are lost on every restart: a Lorenz σ, painted Life cells, pathfinding walls, the sorting
list. Nothing can be handed to someone else. Scope 6 adds sheet files: a CAD-style text format
(`.fwb`) that keeps the layout, every tool's settings, the camera, the canvas settings and the
panels. They open and save through a File menu, shortcuts, commands, drag-and-drop and a Recent
list, and three built-in example sheets ship with the app.

**Branch:** `feat/sheet-files`, created from the updated `main`. One PR at the end. The user reviews
and merges.

**Fixed user decisions (from the roadmap):**
- The extension is `.fwb`, in CAD-style sections like DXF.
- A sheet saves the layout, each tool's settings, the camera and grid settings, and the theme and
  panels.
- A newer-version or damaged file is refused with a message, and the board is left untouched.
- Unknown tools, sections and keys are skipped, with warnings.
- Files open through the File menu with shortcuts, the command line, drag-and-drop, and Recent
  files (desktop only).
- `rfd` is the only new dependency.

## Rulings made while planning

1. **Shared data gets its own `SECTION SHARED`.** The roadmap's "write it through the first
   viewport" is replaced.
   - The sorting list and the pathfinding grid / MST graph belong to the board (`ToolEnv`), not to
     a viewport.
   - `ToolEnv::save_state()` / `load_state()` write them once, and they load *before* the viewports
     are built.
   - Cost if wrong: one section's code moves into the viewports.
2. **A tool saves what the user sets:**
   - sliders, toggles, modes, seeds;
   - dragged or edited points;
   - painted cells and walls;
   - whether it is playing.

   It does not save animation clocks, caches, hover or drag state.
3. **Field-by-field leniency through one helper.** `state::merge(current, json)` tries each field
   on its own, so one bad field never discards the rest. Each tool then re-clamps to its slider
   ranges, so a hostile sheet (`cols 1e12`) cannot hang the app or exhaust memory.
4. **Opening is one undo step.**
   - It records a `Group` of `Removed` (old viewports) and `Added` (new ones). New viewports get
     fresh uids; the titles are kept.
   - Undo restores the old viewports. The shared list and grid stay as the sheet set them.
   - The camera, settings and panels are not part of history.
   - No "discard changes?" prompt is needed, because Open can be undone.
5. **The `•` (unsaved changes) compares text.** Slider moves are not in history, so the dirty flag
   compares the sheet's text, without CAMERA and UI, against the last saved or opened text. It is
   rechecked at most every 0.5 s.
6. **Saving on the web downloads `name.fwb`**, in the same way as `export::save`. Opening on the
   web uses `rfd::AsyncFileDialog::pick_file` with an inbox that is polled every frame. On native,
   `rfd::FileDialog` is synchronous.
7. **`save` with no path and no name** writes `exports/sheets/untitled.fwb` on native, like the
   other exports. Ctrl+S without a path opens Save As.
8. **Grids are written as row strings.** Life cells use `.` and `#`. The pathfinding grid uses
   `.` (open), `#` (wall), `~` (mud), `S` (start) and `G` (goal).

## Global constraints

- **Gate before every commit:** `cargo fmt --check && cargo clippy --all-targets -- -D warnings
  && cargo test`. The scratchpad `gate.sh` prints `GATE OK`.
- The wasm check must pass: `cargo check --lib --target wasm32-unknown-unknown`.
- **`rfd = "0.17"` with default features** (xdg-portal, so no GTK headers on Linux). No other new
  dependencies.
- **Old saves keep loading:**
  - `board-v1` saves keep loading, and new fields are `#[serde(default)]`.
  - `Saved` keeps its key.
  - Tests use a fixture of a `board-v1` JSON from before this scope.
- `cad/fwb.rs` and `cad/sheet.rs` stay UI-free and have unit tests. The workbench still never names
  a specific formula; it reaches them only through the hooks.
- **Palette colors only.** All UI text must pass the glyph test: no `→` and no `…` in menu labels;
  use `Open...`.
- README and CLAUDE.md are updated in the last task. CLAUDE.md's "Persistence" paragraph currently
  says "A formula's own state is **not** saved" and must be rewritten.

## Review focus (each line gets its test in the named task)

1. **Hostile values in a sheet:**
   - Values like `param cols 1e12`, `param n -5`, `param points "x"` and `null` for a number.
   - Expected: the value is clamped or ignored, the other fields still load, nothing panics, and
     no huge allocation happens. [Tasks 2, 3]
2. **Damaged files** are refused with a line number, and the board, camera and history stay
   unchanged. Covered cases: truncated, binary, empty, `FWB 99`, missing `ENDSEC`, nested
   `SECTION`. [Tasks 4, 5]
3. **A sheet with an unknown tool, section or key** loads the rest and reports one warning per
   problem. [Task 5]
4. **Undo after Open** brings back the previous board. Redo opens the sheet again. The shared
   sorting list is not swapped out from under the old viewports. [Task 5]
5. **Recent files:**
   - A duplicate path moves to the front, and the list stops at 8.
   - A recent file that has since been deleted gets a message and is removed from the list, and
     the board is kept. [Task 6]

---

## File map

| File | Role |
|---|---|
| `src/state.rs` (new) | `merge<T>(current, &Value) -> T`, plus `rows_to_strings` / `strings_to_rows` grid helpers |
| `src/formula.rs` | Doc comments on `save_state` / `load_state` (the hooks exist since Scope 3) |
| `src/formulas/**` | `save_state` / `load_state` for every tool through a private `State` struct |
| `src/formulas/mod.rs` | `ToolEnv::save_state` / `load_state`; generic tests over `TOOLS` |
| `src/cad/board.rs` | `ViewportModel.state`, `to_model` / `from_model` call the hooks, `Board::add_model` |
| `src/cad/fwb.rs` (new) | The generic section codec: `Doc`, `Section`, `Entry`, `read`, `write`, `FwbError` |
| `src/cad/sheet.rs` (new) | `Sheet`, `UiState`, `to_text`, `from_text` (with warnings), `migrate` |
| `src/cad/files.rs` (new) | Current path, dirty tracking, the Recent list, dialogs (native sync, web inbox), drag-and-drop |
| `src/cad/canvas.rs` | `Workspace::{sheet, open_text, save_to}`, the `open` and `save` commands, shortcuts |
| `src/cad/chrome.rs` | File menu: Open... / Save / Save As... / Recent / Examples |
| `src/app.rs` | `Saved` gains `shared` and `file`; mirrors `UiState`; window title; key `recent-sheets` |
| `src/web.rs`, `index.html` | `set_title`; Ctrl+S and Ctrl+O `preventDefault` |
| `build.rs`, `assets/sheets/*.fwb` | Embedded example sheets (`SHEETS` table) |
| `Cargo.toml` | `rfd = "0.17"` |

---

### Task 1: State plumbing (6.1, part 1)

**Produces:**
```rust
// src/state.rs
/// `current` with every top-level field of `v` that deserializes on its own applied.
pub fn merge<T: Serialize + DeserializeOwned>(current: &T, v: &Value) -> T;
/// Grid rows as strings: `cell(x, y)` picks the character.
pub fn grid_rows(w: usize, h: usize, cell: impl Fn(usize, usize) -> char) -> Vec<String>;
// src/formulas/mod.rs
impl ToolEnv { pub fn save_state(&self) -> serde_json::Map<String, Value>; pub fn load_state(&mut self, v: &serde_json::Map<String, Value>); }
// src/cad/board.rs
pub struct ViewportModel { …, #[serde(default, skip_serializing_if = "Option::is_none")] pub state: Option<Value> }
impl Board { pub fn add_model(&mut self, m: &ViewportModel, fresh_uid: bool) -> Option<u64>; }
```
- **`merge`:**
  - Serialize `current` into an object.
  - For each key in `v` that the object already has, set it and try `from_value::<T>`. Keep it on
    success and revert on failure.
  - If `v` is not an object, return a clone of `current`.
- **`ToolEnv`:**
  - `sorting` holds `{n, dist, seed, skin}`.
  - `grid` holds `{w, h, rows}` (Ruling 8).
  - `graph` holds `{points, seed}`, and the edges are rebuilt.
  - `load_state` clamps `n` to the sorting size slider range. Grid rows of the wrong size are
    ignored. Loading bumps `generation`, so viewports re-record.
- **Board:**
  - `to_model` fills `state` from `save_state()`.
  - `from_model` uses `add_model(m, false)`, which calls `load_state`.
  - `app.rs` `Saved` gains `#[serde(default)] shared: Option<Map>`. It is loaded into `board.env`
    *before* `from_model`, so the formulas are built on the restored shared data.

**Tests:**
- [ ] `merge`:
  - `{"a": 2, "b": "x"}` onto `{a:1,b:true}` gives `{a:2,b:true}`;
  - `json!(42)` changes nothing;
  - unknown keys are ignored.
- [ ] A fixture `board-v1` JSON without `state` or `shared` (copied from a save made before this
  scope) deserializes, and `from_model` builds every viewport.
- [ ] `ToolEnv` round trip: shuffle the list and paint a wall, then save, load into a fresh env and
  save again; both saves are equal. A grid with wrong-sized rows is ignored, and so is `n: 1e12`.

Commit: `Save each viewport's state and the shared sorting and graph data`.

### Task 2: Per-tool state, part A (non-session tools)

**The pattern for every tool:**
- A private `#[derive(Serialize, Deserialize)] struct State { … }` holds the saved fields.
- Enums gain `#[derive(Serialize, Deserialize)] #[serde(rename_all = "snake_case")]`.
- The hooks:
  ```rust
  fn save_state(&self) -> Option<Value> { serde_json::to_value(self.state()).ok() }
  fn load_state(&mut self, v: &Value) { self.apply(state::merge(&self.state(), v)) }
  ```
- `apply` clamps each number to its slider range, drops non-finite values, caps list lengths at
  the tool's maximum, and regenerates derived data (fits, caches, textures).

**Fields saved:**

| Tool | Saved fields |
|---|---|
| polar-to-cartesian | mode, r, theta, petals, playing, speed |
| perspective-projection | focal, distance, yaw, pitch, spinning |
| lerp | t, playing, speed, v0, v1, ends |
| cubic-bezier | points, t, playing, construction, speed |
| fourier-epicycles | animal (by name), terms (`target`), speed, playing, guide |
| lissajous | amp_x, amp_y, a, b, delta, speed, playing, drift, drift_rate |
| force-layout | seed, k, rest, kr, gravity, damping, running, show_forces |
| min-max-normalization | base, outlier, seed |
| mandelbrot | cx, cy, span, max_iter |
| flow-field | seed, count, zoom, turns, speed, fade, evolve, coloring, show_field, playing |
| game-of-life | cols, pattern, rate, playing, seed, wrap, cells (rows of `.#`) |
| lorenz | sigma, rho, beta, count, speed, trail, yaw, pitch, spinning, playing |
| euclidean-distance | mode, points, seed, k, auto |
| rotation-matrix | theta, spinning, point |
| voronoi | seeds (positions), count, style, drift, relax, delaunay, show_seeds |
| matrix-transform | chain (`[[a,b],[c,d]]` each), selected, figure, snap, animating |
| linear-regression | points, seed, noise, residuals, squares |
| ease-out-cubic | duration, looping, eased, seed |
| relative-luminance | selected, linearize, naive |
| quadratic-formula | a, b, c, view |
| pythagorean-proof | a, b, t, playing, mode |
| euler-identity | theta, terms, playing |

`expression` already has its hooks; check that it passes the generic tests.

**Generic tests in `formulas/mod.rs`** (they run over every tool whose `save_state` is `Some`;
Task 3 makes that every tool):
- [ ] `state_round_trips`: make, save `s1`, load `s1` into a fresh instance, save `s2`, and check
  `s1 == s2`. Every top-level key matches `[a-z_][a-z0-9_]*`, because keys become sheet `param`
  names.
- [ ] `flipped_toggles_survive`: for every top-level bool field, flip it, load into a fresh
  instance, and check the saved value is flipped.
- [ ] `hostile_state_is_clamped` (Review Focus 1):
  - For every top-level number field, try `1e12`, `-1e12`, `null` and `"x"` in turn.
  - Load each into a fresh instance, run one frame through `every_tool_runs_on_the_canvas`'s
    helper at `min_size`, and check the field saved afterwards is finite.
  - The whole test finishes in under 5 s in debug.
- [ ] Per-module tests for structured state:
  - life: painted cells survive, and rows of the wrong width are ignored;
  - distance and regression: dragged points survive;
  - matrix: a chain of 3 survives;
  - fourier: an unknown animal name keeps the current animal.

Commit: `Save and restore the settings of every standalone tool`.

### Task 3: Per-tool state, part B (session tools)

**Fields saved:**

| Tool | Saved fields |
|---|---|
| 7 sorting tools | speed, playing (the list and skin are in SHARED) |
| sorting-race | chosen (algorithm names), speed, running |
| bfs, dfs, dijkstra, a-star | speed, playing, mud (brush) (the grid is in SHARED) |
| binary-search-tree | keys (`history`), balance, seed, key, speed, playing |
| kruskal, prim | n, speed, playing (the graph is in SHARED) |

- The BST rebuilds from `keys`, capped at its maximum node count.
- The race ignores unknown algorithm names.

**Tests:**
- [ ] New test `every_tool_saves_state`: `save_state()` is `Some` for every tool in `TOOLS`. This
  turns all the Task 2 tests on for all tools.
- [ ] BST: inserted keys survive. A list of 10 000 keys is capped.
- [ ] Race: `["bubble", "nope"]` gives one lane.

Commit: `Save and restore sorting, graph and tree tools`.

### Task 4: The `.fwb` codec (6.2a, `src/cad/fwb.rs`)

**Produces:**
```rust
pub const VERSION: u32 = 1; pub const SHEET_EXT: &str = "fwb";
pub struct Doc { pub version: u32, pub sections: Vec<Section> }
pub struct Section { pub name: String, pub line: usize, pub entries: Vec<Entry> }
pub struct Entry { pub key: String, pub rest: String, pub line: usize }   // rest = raw text after the key
impl Entry { pub fn value(&self) -> Value; pub fn split_name(&self) -> Option<(&str, Value)>; } // "param a 3"
pub fn encode(v: &Value) -> String;   // scalar → JSON; array of ≥2 numbers/bools → space-separated; else compact JSON
pub fn decode(s: &str) -> Value;      // one JSON value, else whitespace-split JSON scalars as an array, else the raw string
pub fn write(doc: &Doc) -> String;    // "FWB 1", two-space indented entries, blank line between sections
pub fn read(text: &str) -> Result<Doc, FwbError>;
pub enum FwbError { NotASheet, NewerVersion(u32), Malformed { line: usize, message: String } }
impl Display for FwbError  // "line 14: SECTION VIEWPORT is missing ENDSEC"
```
**Grammar:**
- Line 1 is `FWB <int>`.
- Indentation is ignored. Blank lines and lines starting with `;` are skipped. `\r\n` is accepted.
- `SECTION <NAME>` … `ENDSEC`.
- An entry is `key rest`, where the key matches `[a-z_][a-z0-9_]*`.
- An array of strings is always compact JSON, so a quoted string with spaces can never be split.

**Tests:**
- [ ] `encode` / `decode`:
  - `[0,0]` ↔ `0 0`;
  - `[3]` ↔ `[3]`;
  - `"a \"b\" c"`;
  - nested arrays, `true false`, `null`, `[]`;
  - an object;
  - `decode(encode(v)) == v` for 200 seeded random JSON values (scalars, arrays, objects).
- [ ] The example from the roadmap reads into 5 sections. `write(read(x))` read again is equal.
- [ ] Comments, blank lines, CRLF and leading spaces are handled.
- [ ] Errors (Review Focus 2):
  - an empty file, binary bytes (`"\0\u{1}"`), or a missing header → `NotASheet`;
  - `FWB 99` → `NewerVersion(99)`;
  - a missing `ENDSEC` (line of the `SECTION`), a nested `SECTION`, `ENDSEC` without a section, a
    key outside a section, or a bad key `A-b` → `Malformed` with the right line;
  - each `Display` text names the line.

Commit: `Add the .fwb section codec`.

### Task 5: Sheets and applying them (6.2b, `src/cad/sheet.rs` + `Workspace`)

**Produces:**
```rust
pub struct UiState { pub dark: bool, pub browser: bool, pub inspector: bool, pub tab: Tab, pub lang: Lang }
pub struct Sheet { pub title: Option<String>, pub camera: Camera, pub settings: Settings, pub ui: UiState,
                   pub shared: Map<String, Value>, pub viewports: Vec<ViewportModel> }
pub fn to_text(s: &Sheet) -> String;
pub fn from_text(text: &str) -> Result<(Sheet, Vec<String>), FwbError>;   // warnings
fn migrate(doc: Doc) -> Doc;   // identity for version 1; the hook for later versions
// canvas.rs
pub ui: UiState, pub ui_changed: bool            // mirror that app.rs syncs both ways
pub fn sheet(&self) -> Sheet;
pub fn open_text(&mut self, name: &str, text: &str) -> bool;  // say() the result and warnings
```
**Sections:**
- `HEADER`: `app`, optional `title`.
- `CAMERA` and `SETTINGS`: through serde, one entry per field. A missing field keeps its default.
- `UI`: `theme dark|light`, `browser`, `inspector`, `tab properties|docs|code|derive`,
  `lang python|typescript|rust`.
- `SHARED`: one entry per `ToolEnv` field.
- `VIEWPORT` (in z-order): `uid`, `tool`, `title`, `rect`, `locked`, `hidden`, then
  `param <name> <value>` for each top-level field of the state.

**Lenient reading:**
- An unknown section or key gives the warning `line N: unknown key "foo" in VIEWPORT`.
- An unknown tool skips that viewport with `skipped VP-04 (unknown tool "foo")`.
- A value of the wrong type keeps the default and gives a warning.

**`open_text`:**
1. Parse fully first. On error, `say("Could not open name: line 14: …")`, return false, and change
   nothing.
2. Otherwise:
   - load `shared` into `board.env`;
   - remove every viewport;
   - add the new ones with `add_model(m, true)`;
   - push a single `Edit::Group([Removed…, Added…])`;
   - set the camera, settings and `ui` (`ui_changed = true`);
   - clear the selection;
   - `say("Opened name")`, plus `", N warnings"` and each warning.

**Tests:**
- [ ] A round trip with every tool on one sheet, each with one non-default value: `from_text(to_text(s))`
  gives an equal sheet and no warnings.
- [ ] The roadmap example parses exactly (field values asserted).
- [ ] An unknown tool, section and key give exactly 3 warnings, and the rest loads (Review Focus 3).
- [ ] An empty sheet (`FWB 1` only) is valid and loads no viewports, with defaults.
- [ ] `open_text` with bad text leaves the board's models, camera and `history.can_undo()`
  unchanged.
- [ ] `open_text` then `undo` gives the old models. `redo` gives the sheet's models (Review
  Focus 4).

Commit: `Read and write whole sheets and open them as one undo step`.

### Task 6: Files, commands, dirty tracking, drag-and-drop (6.3a, `src/cad/files.rs`)

**Produces:**
```rust
pub const RECENT_MAX: usize = 8;
pub fn push_recent(list: &mut Vec<String>, path: &str);    // dedup, front, cap
pub fn file_name(path: &str) -> String;                     // "lesson.fwb"
pub struct Files { pub path: Option<String>, saved: Option<String>, pub dirty: bool, checked: f64,
                   pub recent: Vec<String>, #[cfg(wasm)] inbox: Rc<RefCell<Option<(String, Vec<u8>)>>> }
// Workspace
pub files: Files;
pub fn open_path(&mut self, path: &str) -> bool;           // native: read, 8 MB cap, UTF-8 else NotASheet
pub fn save_to(&mut self, path: Option<&str>) -> bool;     // native write / web download; updates path, saved, recent
```
- **Commands** (added to `help`):
  - `open <path>` works on native. Without a path, or on the web, it sets
    `Request::OpenDialog`, which the chrome handles.
  - `save [name]` follows Ruling 7. A name without `.fwb` gets it appended.
- **Dirty tracking:** `Workspace::show` runs `files.check(now, || dirty_key(&self.sheet()))` every
  0.5 s. `dirty_key` is `to_text` with the camera and UI reset to their defaults.
- **Drag-and-drop:** read `ctx.input(|i| i.raw.dropped_files)`. Use `path` on native and `bytes`
  on web. Only the first `.fwb` file is opened; anything else gets `"name is not a .fwb sheet"`.
- **Recent:** `open_path` of a missing file says so and removes it from `recent` (Review Focus 5).

**Tests:**
- [ ] `push_recent`: a duplicate moves to the front, a 9th entry drops the oldest, `remove` works.
- [ ] `save` on native in a temp dir (`std::env::set_current_dir` is not safe in parallel tests, so
  test `save_to(Some(tmp/x.fwb))` instead) writes text that `open_path` reads back to the same
  models. `save name` resolves to `exports/sheets/name.fwb` (a pure `resolve_save_path` function
  is tested; no write).
- [ ] Dirty: `false` after a save, `true` after a placement or a `load_state` change, `false` again
  after undo back to the saved models. A camera pan does not make it dirty.
- [ ] Dropping (feed `open_bytes(name, bytes)`, the shared path of drop and web open) a valid sheet
  replaces the board. A non-sheet keeps the board and reports the error.
- [ ] `open_path` of a missing file removes it from `recent`.

Commit: `Open and save sheets from commands and by dropping files`.

### Task 7: Dialogs, File menu, shortcuts, title (6.3b)

- `Cargo.toml`: `rfd = "0.17"` (both targets).
- **File menu** in `chrome.rs`:
  - New sheet; Open... (Ctrl+O); Save (Ctrl+S); Save As... (Ctrl+Shift+S);
  - Recent ▸ (native only, with "Clear recent");
  - Examples ▸ (Task 8);
  - Demo sheet.
- **Shortcuts** in the `canvas.rs` key block: `cmd_shift S` before `cmd S`, and `cmd O`.
- **Dialogs** in `files.rs`:
  - native `rfd::FileDialog::new().add_filter("Formula sheet", &["fwb"])` with `.pick_file()` /
    `.save_file()` (default name `untitled.fwb`);
  - web `rfd::AsyncFileDialog` with `pick_file()`, then `handle.read().await` into the inbox,
    polled by `Workspace::show`.
- **Title:**
  - `app.rs` builds `"{name}{ •}  - Formula"` and sends `ViewportCommand::Title` only when it
    changes. On wasm it calls `web::set_title` (document.title).
  - `Saved` gains `#[serde(default)] file: Option<String>`. The recent list is read and written
    under the eframe key `"recent-sheets"`.
- **`index.html`:** add `"s"` and `"o"` to the Ctrl `preventDefault` list.
- **Tests:**
  - `menu_labels_have_glyphs` (or extend `all_text_has_glyphs` with the menu strings);
  - the `help` text mentions `open` and `save`;
  - `Request::OpenDialog` is set by `open` with no argument.
  - Manual checks for the dialogs themselves are in Verification.

Commit: `Add the File menu with Open, Save, Save As and Recent`.

### Task 8: Example sheets, docs (6.4)

- **`assets/sheets/`:**
  - `linear-algebra.fwb`: matrix-transform, rotation-matrix, perspective-projection and lerp,
    arranged and with chosen params.
  - `sorting-showdown.fwb`: 3 sorting tools, plus the race on a 60-element nearly-sorted list in
    SHARED.
  - `pathfinding-lab.fwb`: bfs, dijkstra and a-star on a hand-drawn maze in SHARED.

  Each has `title` in HEADER and was written by the app and then hand-tidied.
- **`build.rs`** also scans `assets/sheets/*.fwb` and writes `$OUT_DIR/sheets.rs` with
  `pub static SHEETS: &[(&str /*file*/, &str /*text*/)]`. `src/cad/sheet.rs` uses
  `include!`s it, and `examples()` returns `(title, text)` pairs.
- **File → Examples** shows them by title. Picking one runs `open_text`, so it is undoable.
- **Tests:** every example parses with **zero warnings**, every tool id exists, and every title is
  non-empty and passes the glyph test.
- **Docs:**
  - README: a "Sheet files" section (format sample, menu, shortcuts, commands, drag-and-drop, Recent),
    and `open` / `save` in the commands list.
  - CLAUDE.md: rewrite "Persistence" (formula state is saved now; `ToolEnv` state; how a tool
    adds its `State`). Add a "Sheet files" paragraph (`fwb.rs` codec / `sheet.rs` mapping /
    `files.rs` I/O, leniency rules, bump `VERSION` + `migrate` on format changes). Add an "Adding a
    formula" step 9: implement `save_state` / `load_state`, which the tests enforce.

Commit: `Add example sheets and document sheet files`.

---

## Verification (before the PR)

1. Run the gate and check it prints `GATE OK`. Then run
   `cargo check --lib --target wasm32-unknown-unknown`.
2. Web build plus headless Chromium (`scratchpad/scripts/`):
   - open each example from File → Examples and screenshot it;
   - drop a `.fwb` file (Playwright `DataTransfer`) and check the board changes;
   - drop a `.txt` file and check the error appears in the command log;
   - press Ctrl+S and check a download of `untitled.fwb` happens;
   - confirm there are no page errors.
3. Native `cargo run --release`:
   - Open..., Save As... and Save with the macOS dialogs;
   - reopen from Recent;
   - edit a slider and check the title shows `•`, then save and check it clears;
   - quit and relaunch, and check the board-v1 restore keeps each tool's params.

   The user does the native smoke test if dialogs can't be driven here.
4. Push, then open the PR. The user reviews and merges.

---

## Rulings made while building

- **Floats parse exactly.** serde_json's `float_roundtrip` feature (of an existing dependency)
  makes f64 values come back bit for bit. Without it, `-983.8800048828125` read back one ulp off.
- **f32 settings are written short.** `state::to_value` goes through `to_string`, so a sheet says
  `0.78`, not `0.7799999713897705`.
- **Animated values are saved only while paused.** Examples: lerp, bezier and pythagoras `t`;
  polar and euler `θ`; a spinning `yaw`; Lissajous `delta` while drifting; Life cells while
  running; Voronoi seeds while moving; easing's seed while looping. Each is an `Option` field
  skipped while running. Replay tools don't save `playing`. Without this, any running animation
  would mark the sheet unsaved. A test runs every tool for 30 frames and checks its saved state
  doesn't change.
- **`Formula::state_params`** lists every field a formula loads, including the skipped `Option`s,
  read from the serde struct through a small field-recording deserializer. Sheets warn about any
  `param` not on that list.
- **`state::merge` also takes fields that the current state leaves out.** Unknown fields
  deserialize and are ignored.
- **Bare words and default shared data.**
  - Simple words are written unquoted (`tool lerp`, `theme dark`), as in the roadmap's example.
  - SHARED holds only data that differs from a new board's.
  - When reading, missing SHARED entries are a new board's, so opening a sheet gives the same board
    every time.
- **Example sheets open with `fit_all`** and are not a current file, so Save asks where to save.
- **Force layout** saves its seed and regenerates the graph only when the seed changes. Shuffled
  positions are not saved; the layout settles again anyway.
