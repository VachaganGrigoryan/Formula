# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Formula is a CAD-style "formula workbench" written in Rust on egui/eframe 0.36 (edition 2024). It
runs natively and in the browser (wasm). Every formula is a *tool* that is placed on an infinite
drafting canvas as a live, resizable viewport. The Inspector shows its controls, reference docs, and
exportable Python/TypeScript/Rust code. `README.md` is the user-facing manual (features, controls, layout).

## Commands

```sh
cargo run --release                          # native app
cargo run --release --features native-audio  # with sorting sounds via cpal (Linux: libasound2-dev)
cargo test                                   # all tests (~170, fast)
cargo test <name_substring>                  # one test, e.g. cargo test every_tool_runs_on_the_canvas
cargo test formulas::sorting                 # one module's tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Web build (the `wasm-bindgen` CLI version must match the `=0.2.129` pin in Cargo.toml/Cargo.lock):

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/release/formula.wasm
python3 -m http.server   # index.html loads ./pkg/formula.js; open http://localhost:8000/#<tool-id>
```

The crate is both `cdylib` (wasm) and `rlib`. `src/main.rs` is the native entry. `src/web.rs`
(`#[wasm_bindgen(start)]`) is the browser entry. `web.rs` also syncs the selected tool to the URL
`#id` so links can be shared. Code that is only for wasm or only for native goes behind
`cfg(target_arch = "wasm32")`.

## Architecture

**The two halves.** `src/cad/` is the workbench (canvas, panels, camera, selection, undo).
`src/formulas/` holds the formulas. They meet in only two places: the `formula::Formula` trait
(`src/formula.rs`) and the tool catalogue `formulas::TOOLS` (`src/formulas/mod.rs`). The workbench
never knows about specific formulas. It iterates `TOOLS`, and each `Tool.make(&ToolEnv)` builds a
fresh `Box<dyn Formula>` per viewport.

**Rendering model.** `cad/canvas.rs` (`Workspace`) draws every viewport's formula at its natural
size, where one world unit is one point, onto a *world layer* that egui scales and translates as a
whole (`ctx.set_transform_layer` with `camera.transform`). The workspace chrome (frames, grips,
guides, crosshair) goes on a second, untransformed overlay layer. Formulas therefore never deal with
zoom. They just `paint` into the `rect` they are given.

**UI-free cores.** The logic lives in modules that don't touch egui and have unit tests:
`cad/camera.rs`, `cad/board.rs` (viewports, z-order, selection, smart-guide `align`, saved model),
`cad/history.rs`, `formulas/sorting/algorithms.rs`, `formulas/fourier/math.rs`. Within each formula
module, keep the math in plain functions with tests, separate from `paint`/`controls`.

**Shared state between viewports.** `ToolEnv` carries state that instances on one board share.
`sorting::Session` is one shared list (and audio output) per sheet, so a shuffle shows up in every
sorting viewport and in the race. `graphs::Session` is one pathfinding grid and one MST graph per
sheet, so every search or MST viewport works on the same input.

**Step-through algorithms.** `formulas/replay.rs` (`Replay`, `Player`, `transport`, `transport_keys`)
plays any recorded op log; sorting, pathfinding and MST use op logs, the BST replays snapshot frames
through `FrameIndex`. `ToolEnv.graphs` shares one grid and one graph per board.

**Persistence.** `app.rs` saves a `Saved` struct under the eframe storage key `"board-v1"`. It
holds the viewport layout (`board::ViewportModel`: uid, tool id, rect, title, locked, hidden), the
camera, settings, theme, panel visibility, and the inspector tab/lang. A formula's own state is
**not** saved. If you change these serialized structs, keep old saves loadable or bump the key.

**Exports.**
- `src/latex.rs` converts `MathText` equations to LaTeX. Function words become operators, Greek
  letters become commands, and English prose goes in `\text{}`. `latex_snippet` adds a `where` line
  with `Formula::params`.
- `cad/notebook.rs` builds an nbformat 4.5 notebook: doc overview, LaTeX, an ipympl setup cell,
  and `main.py`.
- Both use the *live* instance via `Inspector::context_formula`. The `latex` and `ipynb` commands
  do the same.
- Commands can't reach egui, so they put text in `Workspace::clipboard`, which the canvas copies on
  the next frame.

**Expressions.** `src/expr/` is the in-repo expression engine (no crate): a lexer, a Pratt parser
with a depth cap of 256, `eval`, `print` (canonical text that re-parses to the same tree),
`to_math` (a `MathText` for the Inspector and LaTeX) and `to_code` (numpy / TypeScript / Rust).
Letters are single-letter variables (`ab` = a·b, `k_r` is one subscripted name), except for
function names, Greek names (`theta` → `θ`) and the constants `pi` and `e`. The `expression` tool
uses it. Its content files are templates whose `@PARAMS` / `@EXPRESSIONS` regions
`Formula::export_code` regenerates, and the Code tab and the notebook prefer `export_code` over
the static file. A test keeps the static files equal to the default instance's export. The
`plot <expr>` command places a plotter through `Formula::load_state`. `save_state` / `load_state`
are no-ops for every other tool until sheet files arrive.

**Derivations.** The Inspector's Derive tab shows `derive::all_derivations(f)`. The first entry is
"Plug in current values", built from `Formula::substitutions` (each a left-hand side, a source
string parsed with `expr`, and the variable values). `derive::substitution_steps` substitutes the
values, then computes the innermost-leftmost operation on numbers, one per step, until one number
is left; negated numbers fold into the step that made them. After it come the authored
`Formula::derivations` (`Step` = a `MathText` and why it follows). Every step's text is covered by
the glyph test and the ASCII-LaTeX test. The proof tools live in `formulas/proofs/` under the
`PROOFS` category.

**Embedded content.** `build.rs` scans `assets/content/<tool-id>/` and generates
`$OUT_DIR/content.rs` with an `include_str!` for each `doc.md`, `main.py`, `main.ts`, `main.rs`.
`src/content.rs` serves it. Adding content needs no code change.

## Adding a formula (the tests enforce all of this)

1. Add a module under `src/formulas/` and implement `Formula`. Required: `id`, `name`, `category`,
   `summary`, `equations` (built with `MathText`), `paint`, `controls`. Optional hooks: `notes`,
   `details`, `stage_height`, `stage_sense` + `interact`, `is_animating`, `min_size`, `handle_keys`,
   `wants_scroll`, and `params`. `params` reports slider values named as in the equations, for
   example `"θ"` or `"k_r"` (an underscore starts a subscript).
2. Add a `tool!(id, name, CATEGORY_CONST, "ABC" alias, new::<T>)` entry to `TOOLS`. The library and
   toolbar group tools by category in registration order.
3. Add an icon arm `"<id>" => { … }` in `cad::icons::tool` (`src/cad/icons.rs`). The
   `every_tool_has_its_own_icon` test greps the source for that exact pattern.
4. Add `assets/content/<id>/` with all four files:
   - `doc.md` must start with `# <Tool name>` and have exactly the ten `## ` headings in
     `content::SECTIONS`, in order.
   - The code files must be **pure ASCII**.
   - There must be no directories for unknown ids.
5. Every equation must convert to ASCII LaTeX. If you add a new symbol, map it in
   `latex::symbol` (`latex::tests::every_equation_converts_to_ascii_latex`).
6. All UI text must render with egui's bundled font. `formulas::tests::all_text_has_glyphs` fails
   on characters such as ′ ∈ → ᵢ ᵣ or combining accents (x̄).
7. A new category constant needs a short toolbar name in `cad::chrome::short`
   (`every_category_has_a_toolbar_name`); otherwise it shows up as "Other".
8. `canvas::tests::every_tool_runs_on_the_canvas` places every tool at its default size and its
   `min_size` and runs frames with input, so `paint` must not panic at the minimum size.

A slider with `step_by` must also have `.clamping(egui::SliderClamping::Edits)`. Under the default
`Always`, egui re-applies the step to the bound value on every draw. That rounded Lorenz's β = 8/3 to
2.67 and froze Lissajous's δ drift while its controls were visible.
`formulas::tests::showing_controls_leaves_parameters_unchanged` checks every tool.
`matrix::tests::dragging_the_i_tip_on_the_canvas_edits_the_matrix` shows how to drive a drag through
the real `Workspace`, one egui frame per event.

Use `Palette` colors (`x`, `y`, `trace`, `fill`, …, from `formula.rs`) instead of hard-coded colors,
so formulas match in both the light and the dark theme. Helpers for draggable handles, gradients
and polygon fill are in `src/draw.rs`. Seeded randomness is in `src/rng.rs`.

## Sorting

`sorting/algorithms.rs` runs each algorithm once inside a `Recorder` that logs every operation
(`Compare`, `Swap`, `Write`, `Sorted`, markers, `Enter`/`Exit`). The viewport replays that log and
seeks backwards by replaying from the start. To add an algorithm:

1. Add an `Algorithm` variant (also in `Algorithm::ALL`).
2. Write the algorithm against `Recorder`.
3. Add its text to `info()` in `sorting/mod.rs`.
4. Register it in `TOOLS` via `sort::<INDEX>` and give it content/icon as above.

Tests check that every algorithm sorts and replays to the same result.
