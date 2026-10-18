# Scope 3: Expression sandbox (roadmap phases 3.1–3.4)

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (the user
> chose one agent per task plus a reviewer for Scopes 2, 3 and 6). Steps use `- [ ]` checkboxes.
> The first execution step copies this plan to `docs/superpowers/plans/2026-10-18-expression-sandbox.md`
> on the branch.

## Context

Scopes 0, 5, 4 and 2, plus fixes #9 and #11, are merged (main `a7939b3`). Scope 3 is next in the
approved order (0→5→4→2→**3**→1→6). Today every tool is a hand-written formula. Scope 3 lets
the user type their own math, for example `y = a sin(bx)`, `r = 1 + cos(3θ)` or `(cos t, sin t)`.
The tool parses it with an in-repo Pratt parser (user decision: no crate), plots it with a live
slider for every free parameter, and exports it as runnable Python, TypeScript or Rust. Scope 1
(substitution steps) later reuses the same engine.

**Goal:** A new `expression` tool ("Expression plotter", alias `EXP`, category CURVES) with up to 6
rows, auto sliders, adaptive sampling that breaks at asymptotes, pan and zoom, inline parse errors,
generated code export, and a `plot <expr>` command.

**Branch:** `feat/expression-sandbox`, created from the updated `main`. One PR at the end, and the
user reviews it before merge.

**Variable rule (user decision, 2026-10-18):** single letters, like Desmos. `ab` = a·b,
`2xy` = 2·x·y, and `a_1` / `k_r` are subscripted names. Known words stay whole (the user said a
mix is fine where needed): function names, Greek names (`theta alpha beta gamma delta lambda mu
omega phi psi rho sigma tau`, plus the Greek glyphs themselves), and the constants `pi`, `e`.

## Rulings made while planning

- **`to_latex` is replaced by `to_math(e) -> MathText`.** It feeds the Inspector equations, and
  through the existing `latex::latex_of`, the LaTeX and notebook exports. One printer instead of
  two. If this is wrong, a direct LaTeX printer can be added later.
- **`Formula::params()` changes to `Vec<(String, f64)>`.** Today it returns `&'static str` names,
  and user-typed names are not static. This is a mechanical `.into()` across 18 impls plus
  `latex::snippet` and the matrix tests. If this is wrong, it only allocates in exports and tests.
- **`save_state` / `load_state` arrive now, not in Scope 6.** They use the exact Scope 6.1
  signatures, with default no-ops. Only `expression` implements them. `plot` uses them, so no
  downcasting is needed, and Scope 6 just fills in the other tools.
- **Errors are drawn in `palette.y`.** The palette has no red, and CLAUDE.md requires Palette
  colors only.
- **Row colors cycle `trace, x, y`.** Rows 4–6 reuse those colors with a dashed stroke.

## Global Constraints

- Gate before every commit: `cargo fmt --check && cargo clippy --all-targets -- -D warnings &&
  cargo test` (scratchpad `gate.sh` prints `GATE OK`).
- Both targets build: `cargo check --lib --target wasm32-unknown-unknown`.
- `src/expr/` has no egui dependency. Math lives in plain functions with unit tests. There are no
  new dependencies.
- The new tool meets the full checklist:
  - a `TOOLS` entry with a unique alias;
  - an icon arm `"expression" =>`;
  - `assets/content/expression/{doc.md,main.py,main.ts,main.rs}`, with the 10 doc sections and
    ASCII-only code;
  - all UI text has glyphs (no ′ ∈ → ᵢ ᵣ, no combining accents);
  - `paint` must not panic at `min_size`;
  - showing the controls leaves `params()` unchanged, so no `step_by` slider without
    `SliderClamping::Edits`;
  - every equation converts to ASCII LaTeX;
  - the notebook test and `plt.show()` test pass.
- Old `board-v1` saves keep loading. No serialized struct changes in this scope.
- README and CLAUDE.md are updated in the last task.

## Review Focus (each line gets its test in the named task)

1. **Hostile input:** `sin(`, `2^^3`, `)`, the empty string, `ж`, 10 000 nested `(`, `sin()`,
   `max(1)` and `2 3` each give a `ParseError` with a span. No panic, no stack overflow (depth
   cap 256). [Task 1]
2. **Non-finite values:** `1/0`, `ln(-1)`, `sqrt(-1)` and `tan` near π/2. Expected: evaluation
   gives ∞ or NaN, and sampling skips the value or breaks the curve. No NaN or huge coordinate
   reaches the painter. [Tasks 2, 4]
3. **Rows edited into errors and back.** Sliders of names that vanish are removed. Names that
   stay keep their values. A broken row draws nothing and shows its error, and the other rows
   still plot. [Task 5]
4. **Generated code with every function, Greek names, subscripts, constants and all three
   definition kinds** is ASCII and runs (Python headless, `tsc --strict`, `cargo check` against
   macroquad). [Task 6 + verification]
5. **`plot` with a bad expression** says the error in the command line and places nothing.
   `plot` with nothing after it prints usage. [Task 7]

---

## File map

| File | Role |
|---|---|
| `src/expr/mod.rs` | Public API re-exports: `parse`, `free_params`, `eval`, `to_math`, `to_code`, types |
| `src/expr/lexer.rs` | `Token { kind, span: Range<usize> }`, letter splitting, known words |
| `src/expr/ast.rs` | `Expr`, `BinOp`, `Func` (name, arity), `Definition`, `ParseError` |
| `src/expr/parser.rs` | Pratt parser, definition heads, depth cap |
| `src/expr/print.rs` | `print(&Expr) -> String` (canonical, re-parseable) and `to_math` |
| `src/expr/eval.rs` | `eval`, constants |
| `src/expr/code.rs` | `to_code(&Expr, Lang)` and `ident(name, Lang)` (Greek → ASCII) |
| `src/formulas/expression/mod.rs` | The `Expression` formula (rows, params, view, paint, controls) |
| `src/formulas/expression/plot.rs` | UI-free: `View`, `sample_explicit`, `sample_curve`, `nice_step` |
| `src/formulas/expression/export.rs` | Template substitution for `export_code` |
| `src/formula.rs` | `params` signature, new hooks `export_code`, `save_state`, `load_state` |
| `src/cad/inspector.rs`, `src/cad/notebook.rs` | Prefer `export_code` over static content |
| `src/cad/canvas.rs` | `plot` command, help text |
| `src/formulas/mod.rs`, `src/cad/icons.rs`, `src/lib.rs` | Registration |
| `assets/content/expression/*` | doc + templates (static file = default export plus marker lines) |

---

### Task 1: Lexer, AST and parser (`src/expr/`, phase 3.1a)

**Produces:**
```rust
pub enum Expr { Num(f64), Var(String), Neg(Box<Expr>), Bin(BinOp, Box<Expr>, Box<Expr>), Call(Func, Vec<Expr>) }
pub enum BinOp { Add, Sub, Mul, Div, Pow }
pub enum Func { Sin, Cos, Tan, Asin, Acos, Atan, Sqrt, Abs, Exp, Ln, Log, Floor, Ceil, Min, Max }
impl Func { pub fn name(self) -> &'static str; pub fn arity(self) -> usize; pub fn from_name(&str) -> Option<Func>; pub const ALL: [Func; 15]; }
pub enum Definition {
    Explicit { var: String, body: Expr },          // y = …, f(x) = …, or a bare expression (var "x")
    Parametric { t: String, x: Expr, y: Expr },    // (x(t), y(t)); t is "t"
    Polar { theta: String, r: Expr },              // r = …; theta is "θ" (input "theta" or "θ")
}
pub struct ParseError { pub span: Range<usize>, pub message: String }   // byte span into the source
pub fn parse(src: &str) -> Result<Definition, ParseError>;
pub fn parse_expr(src: &str) -> Result<Expr, ParseError>;   // body only, used by tests and round-trip
pub fn free_params(def: &Definition) -> Vec<String>;        // sorted, deduped; excludes the independent var, pi, e
pub fn print(e: &Expr) -> String;                           // in print.rs, canonical: "a * sin(b * x)"
```
`Var` names are canonical: Greek words become glyphs (`theta` → `θ`), and subscripts stay `k_r`.
`pi`/`π` and `e` parse to `Var("π")`/`Var("e")` and are treated as constants.

**Grammar:**
- **Precedence**, lowest to highest: `+ -`, then `* /` and implicit multiplication (same level,
  left-assoc, so `1/2x` = (1/2)·x), then unary `-`, then `^` (right-assoc; `-x^2` = −(x²),
  `2^-1` works).
- **Implicit multiplication:** number·ident, ident·ident (letter runs split), `)(`, number·`(`,
  ident·`(` when the ident is not a function.
- **A function without parentheses** takes the following implicit-product chain up to the next
  function name or `+ -`. `sin x` = sin(x), `sin 2x` = sin(2x), `sin x cos x` = sin(x)·cos(x),
  `cos t + 1` = cos(t)+1.
- **Arity:** `min`, `max` take 2 arguments, everything else 1. A wrong count is an error on the
  call span.
- `·` and `×` are accepted as `*`, and `−` as `-`.
- **Definition heads:**
  - `y = E` → Explicit x.
  - `f(v) = E` (any single letter name, any var) → Explicit v.
  - `r = E` → Polar.
  - `( E , E )` at the top level → Parametric t.
  - A bare `E` → Explicit x.
  - Anything else, for example `x = 3`: "Write y = …, r = …, f(x) = … or (x(t), y(t))".
- **Recursion depth cap 256.** Beyond it: `ParseError` "too deeply nested".

**Tests (`src/expr/parser.rs` tests mod):**
- [ ] Precedence table, ≈20 cases comparing `print(parse_expr(src))`:
  - `1+2*3` → `1 + 2 * 3`
  - `-x^2` → `-(x ^ 2)`
  - `2^3^2` → `2 ^ (3 ^ 2)`
  - `2^-1`
  - `2ab x` → `2 * a * b * x`
  - `k_r sin(wx)` → `k_r * sin(w * x)`
  - `1/2x` → `1 / 2 * x`
  - `sin 2x`, `sin x cos x`, `(x+1)(x-1)`, `3(x)`, `a(x+1)`, `cos t + 1`
  - `theta` → `θ`, `2pi`, `x·y`, `x−1`, `max(a, b)`, `e^x`
- [ ] Definitions:
  - `y = sin(x)` → Explicit x.
  - `f(t) = t^2` → Explicit t.
  - `r = 1 + cos(3θ)` and `r = 1 + cos(3theta)` → Polar, same body.
  - `(cos t, sin t)` → Parametric.
  - `sin(x)` → Explicit x.
  - `x = 3` → error.
- [ ] `free_params`: `f(x) = a sin(b x)` → `["a","b"]`; `r = k cos(θ) + pi` → `["k"]`;
  `(cos t, a sin t)` → `["a"]`; `y = e^x` → `[]`.
- [ ] Error spans (Review Focus 1). Assert both the span and a non-empty message:
  - `sin(` → span at the end;
  - `2^^3` → span of the second `^`;
  - `)` → 0..1;
  - empty → 0..0, "type an expression";
  - `ж` → its 2-byte span;
  - `sin()`, `max(1)` → the call span;
  - `2 3` → "missing operator".
- [ ] `"(".repeat(10_000) + "x" + ")".repeat(10_000)` → `Err`, and the test thread does not
  overflow.
- [ ] Round-trip: `parse_expr(&print(&e)) == e` on 500 seeded random ASTs (`crate::rng::Rng`,
  depth ≤ 5, positive literals; negation via `Neg`).

Commit: `Add expression parser: Pratt parser with single-letter variables`.

### Task 2: Evaluation, `to_math` and `to_code` (phase 3.1b)

**Produces:**
```rust
pub fn eval(e: &Expr, env: &dyn Fn(&str) -> Option<f64>) -> f64;   // unknown var -> NaN; π, e built in
pub fn to_math(def: &Definition) -> MathText;                        // e.g. "y = a sin(b x)", ^ as superscript
pub fn to_code(e: &Expr, lang: Lang) -> String;                      // crate::content::Lang
pub fn ident(name: &str, lang: Lang) -> String;                      // "θ" -> "theta", "k_r" -> "k_r", "π" -> np.pi / Math.PI / PI
```
- **Python:** `np.sin(b * x)`, `x ** 2`, `np.log10`, `np.log` for ln, `np.minimum`/`np.maximum`,
  `np.pi`, `np.e`, `np.abs`, `np.floor`, `np.arcsin`.
- **TypeScript:** `Math.sin(b * x)`, `Math.pow(x, 2)` (`**` is avoided because `-x ** 2` is a
  syntax error), `Math.log10`, `Math.log`, `Math.min`, `Math.PI`, `Math.E`.
- **Rust:** `(b * x).sin()`, `x.powf(2.0)`, `x.log10()`, `x.ln()`, `a.min(b)`,
  `std::f64::consts::PI`. Literals print as `2.0`. A literal method receiver prints as `2.0_f64`
  so the type is inferred.
- Parentheses are added only where precedence needs them. The output is always ASCII.

**Tests:**
- [ ] `1/0` → +∞; `ln(-1)` and `sqrt(-1)` → NaN; `2^3^2` = 512; `-2^2` = −4; `pi` ≈ π; an
  unknown var → NaN.
- [ ] Every `Func` evaluates as its `f64` method on a sample input.
- [ ] Codegen exact strings for `sin(b x)` in all three languages: `np.sin(b * x)`,
  `Math.sin(b * x)`, `(b * x).sin()`.
- [ ] Codegen for `-x^2`, `2^x` (Rust `2.0_f64.powf(x)`), `max(a, θ)` and `e^x`. Every result
  is ASCII.
- [ ] For 300 seeded random ASTs (the Task 1 generator) and every lang, `to_code` does not panic
  and its output is ASCII with balanced parentheses.
- [ ] `to_math` for `y = a sin(bx)` has text `y = a sin(b x)`. `latex::latex_of` of it is ASCII
  and contains `\sin`. `x^2` becomes a `Piece::Sup("2")`.

Commit: `Evaluate expressions and print them as math and code`.

### Task 3: Formula hooks (mechanical, cheapest model)

- [ ] `src/formula.rs`:
  - `fn params(&self) -> Vec<(String, f64)>` (was `&'static str`).
  - Add:
    ```rust
    /// Generated code for this instance, preferred over the static content file.
    fn export_code(&self, _lang: Lang) -> Option<String> { None }
    /// The user-set state, for sheets and `plot`. Top-level object fields become `param` lines.
    fn save_state(&self) -> Option<serde_json::Value> { None }
    fn load_state(&mut self, _v: &serde_json::Value) {}
    ```
- [ ] Update all 18 `params` impls (`("θ", v)` → `("θ".into(), v)`), `latex::snippet(params:
  &[(String, f64)])`, and the matrix tests (`p.0.as_str()`).
- [ ] No behavior change. The full gate passes.

Commit: `Let params carry owned names; add export_code and state hooks`.

### Task 4: Plot math (`src/formulas/expression/plot.rs`, UI-free)

**Produces:**
```rust
pub struct View { pub cx: f64, pub cy: f64, pub scale: f64 /* world units per point */ }
impl View {
    pub fn to_screen(&self, rect: egui::Rect, p: [f64; 2]) -> egui::Pos2;   // y up
    pub fn to_world(&self, rect: egui::Rect, p: egui::Pos2) -> [f64; 2];
    pub fn zoom_at(&mut self, rect: egui::Rect, at: egui::Pos2, factor: f64);
    pub fn x_range(&self, rect: egui::Rect) -> (f64, f64); pub fn y_range(&self, rect: egui::Rect) -> (f64, f64);
}
pub fn sample_explicit(f: &dyn Fn(f64) -> f64, x: (f64, f64), y: (f64, f64), px: f32) -> Vec<Vec<[f64; 2]>>;
pub fn sample_curve(f: &dyn Fn(f64) -> [f64; 2], t: (f64, f64), steps: usize, y_span: f64) -> Vec<Vec<[f64; 2]>>;
pub fn nice_step(span: f64, target: usize) -> f64;   // 1, 2, 5 × 10^k
```
(`egui::Rect`/`Pos2` are plain geometry types. That is acceptable here, as in `cad/camera.rs`.)

- **`sample_explicit`:**
  - Start with about `px / 2` uniform samples.
  - Subdivide a segment, up to depth 6, when its midpoint is more than 0.5 px off the chord.
  - Break the polyline at a non-finite value, or when |Δy| > 5 × the y-span and the midpoint does
    not lie between the ends (an asymptote).
  - Clamp the y of drawn points to y-range ± 10 spans, so huge coordinates never reach egui.
- **`sample_curve`** breaks the same way, on non-finite values or jumps.

**Tests:**
- [ ] `tan` on x ∈ [−5, 5], y ∈ [−5, 5], 400 px → exactly 5 pieces. No piece has two neighbouring
  points on opposite sides of ±π/2 or ±3π/2.
- [ ] `1/x` → 2 pieces. `sqrt(x)` on [−1, 1] → 1 piece, with every x ≥ 0. `x^2` → 1 piece.
- [ ] Every point is finite and within ±10 spans (Review Focus 2).
- [ ] `nice_step(10, 5)` = 2, `nice_step(1, 5)` = 0.2, `nice_step(73, 8)` = 10.
- [ ] `View`: `to_world(to_screen(p)) ≈ p`; `zoom_at` keeps the world point under the cursor fixed.
- [ ] A polar circle `r = 1` sampled as a curve closes (first ≈ last).

Commit: `Add adaptive sampling and view math for the expression plotter`.

### Task 5: The `expression` tool (phase 3.2)

**Consumes:** Tasks 1, 2, 4.

**State:**
```rust
pub struct Expression {
    rows: Vec<Row>,
    params: BTreeMap<String, Param>,
    view: View,
    hover: Option<Pos2>,
}
struct Row { src: String, def: Result<Definition, ParseError>, t_range: (f64, f64) }
pub struct Param { pub value: f64, pub min: f64, pub max: f64 }
```
- **Rows:** at most 6. The default rows are `y = a sin(bx)` and `r = 1 + cos(kθ)` with a=2, b=1,
  k=3.
- **`t_range`:** defaults to (0, 2π) for parametric and polar rows, and is editable by two
  DragValues.
- **`Expression::with_rows(&[&str])`** is used by the default and by the tests.
- **`set_row(i, src)`** re-parses, then calls `sync_params()`. Names in no row are removed, names
  that stay keep their values, and new names get value 1 and range −10..10.
- **Paint:**
  - axes plus a grid at `nice_step`, with tick labels;
  - each OK row sampled with Task 4 and drawn in its row color;
  - with a hover, a crosshair, the `(x, y)` readout and a dot on each explicit row at the hover x.
  - Everything clips to `rect`.
- **Interact:** `stage_sense` = click and drag. A drag pans. The wheel or pinch zooms at the
  cursor, only when `response.sense.senses_click()` (copy the `mandelbrot.rs:241-272` pattern).
  `wants_scroll` = true. `is_animating` = false.
- **Controls:**
  - One `TextEdit::singleline` per row. On change, call `set_row`.
  - Under a row with an error, the message in `palette.y`, and a caret line in monospace under the
    error column (char index = `src[..span.start].chars().count()`).
  - Per-row remove button, an "Add row" button (hidden at 6) and a "Reset view" button.
  - One `Slider` per param with `.clamping(SliderClamping::Edits)` and no `step_by`, plus min/max
    `DragValue`s.
- **Hooks:**
  - `params()` → the BTreeMap in order.
  - `equations()` → `to_math` of each OK row (the error rows' source, as plain text).
  - `save_state()` → `{"rows": [src…], "params": {name: [value, min, max]}, "view": [cx, cy, scale]}`.
  - `load_state` accepts the same, leniently: missing fields keep defaults and garbage is ignored.
- **Registration:**
  - `pub mod expression;` and `tool!("expression", "Expression plotter", CURVES, "EXP",
    new::<expression::Expression>)` after `lissajous` (CURVES stays contiguous).
  - `pub mod expr;` in `lib.rs`.
  - An icon arm `"expression" =>`: a `p.curve` of a sine with an `f(x)` feel; copy the
    `"lissajous"` arm style.
  - `assets/content/expression/doc.md`: `# Expression plotter` + the 10 `content::SECTIONS`. It
    covers the grammar, the single-letter rule, sampling and asymptotes, and Pratt parsing
    history (Pratt 1973).
  - For now the static `main.*` are hand-written versions of the default rows, with the Task 6
    markers already in place.

**Tests (`expression/mod.rs`):**
- [ ] `with_rows(["y = a sin(bx)"])` → params `[("a",1),("b",1)]`. Edit to `y = a x` → `[("a",…)]`,
  and `a` keeps its value (Review Focus 3).
- [ ] Edit a row to `sin(` → that row's `def` is `Err`. The other rows' params remain, and
  `paint` at `min_size` does not panic (driven through `every_tool_runs_on_the_canvas`, which
  already covers the default; add a direct paint call with an error row).
- [ ] `save_state` → `load_state` on a fresh instance → equal `save_state`. `load_state` of
  `json!(42)` leaves the defaults.
- [ ] At most 6 rows: `add_row` at 6 is a no-op.
- [ ] Existing global tests pass: icon, docs, ASCII, glyphs, canvas, controls-unchanged, LaTeX,
  notebook.

Commit: `Add the expression plotter tool`.

### Task 6: Generated code export (phase 3.3)

**Templates:**
- Each `assets/content/expression/main.*` holds two regions, delimited by marker lines:
  - Python: `# @PARAMS` … `# @END` and `# @EXPRESSIONS` … `# @END`.
  - TypeScript and Rust: `// @PARAMS` … `// @END` and `// @EXPRESSIONS` … `// @END`.
- The static file is **exactly the default instance's export plus those marker lines**, so it
  runs as-is and stays in sync.
- **Python:**
  - The PARAMS region is the body of `PARAMS = {"a": (2.0, -10.0, 10.0), …}`.
  - The EXPRESSIONS region is the body of `def curves(p, x, t):`. It unpacks the params, then
    returns `[(xs, ys), …]`. Explicit rows are `(x, full(EXPR, x))`, polar rows
    `(R * np.cos(t), R * np.sin(t))`, and parametric rows `(X, Y)`.
  - The template provides sliders, sampling, NaN-masking of jumps, `anim = FuncAnimation(...)`
    and `plt.show()`.
- **TypeScript and Rust:** the same idea, with a `rows` array of closures (`(x, p) => …` /
  `|x: f64, p: &Params| …`) and a self-contained canvas or macroquad viewer, matching the style of
  the existing exports.

**`src/formulas/expression/export.rs`:**
- `pub fn fill(template: &str, regions: &[(&str, String)]) -> String`: replaces each marker
  region, markers included, with the generated text. `Expression::export_code(lang)` calls it with
  `content::code("expression", lang)`.

**Consumers:**
- `inspector.rs` `code()`: compute the source once.
  ```rust
  let src: Option<Cow<'static, str>> =
      self.context_formula(ws).and_then(|f| f.export_code(lang)).map(Cow::Owned)
          .or_else(|| content::code(tool.id, lang).map(Cow::Borrowed));
  ```
  Use it for Copy, Save and the code view; this removes the second `content::code` call at
  `inspector.rs:587`. Extract it as `pub fn code_source(f: Option<&dyn Formula>, id: &str, lang:
  Lang) -> Option<Cow<'static, str>>`, which is testable.
- `notebook::for_formula`: `formula.export_code(Lang::Python)` before falling back to
  `content::code`.

**Tests:**
- [ ] For `y = a*sin(b*x)` with a = 2: the Python export contains `"a": (2.0` (or `a = 2`, per
  the template) and `np.sin(b * x)`, and no `@` marker remains.
- [ ] All three languages are ASCII for rows that cover every `Func`, `θ`, `k_r`, `π`, `e` and all
  three definition kinds (Review Focus 4).
- [ ] **Sync test:** for the default instance and each lang, `export_code(lang)` == the static
  file with its marker lines removed.
- [ ] `code_source(Some(&expr), "expression", Python)` is the generated text.
  `code_source(Some(&lissajous), …)` == `content::code`. `code_source(None, …)` falls back.
- [ ] `notebook::for_formula("expression", &edited)` has a code cell containing the edited
  expression.
- [ ] An `#[ignore]` test `write_sample_exports` writes all three languages for a rich instance to
  `$EXPR_EXPORT_DIR`, for the verification step.

Commit: `Export the plotted expressions as Python, TypeScript and Rust`.

### Task 7: `plot` command, docs (phase 3.4)

- [ ] **`canvas.rs` `run_command`:** add `"plot"`.
  - The rest of the original line, case preserved (`line[4..].trim()`), is parsed with
    `expr::parse`.
  - `Err` → `say("plot: <message> at column N")`, return false, place nothing.
  - Empty → `say("plot <expression>, e.g. plot r = 1 + cos(3θ)")`.
  - `Ok` → `let uid = self.place(formulas::tool("expression").unwrap(), None, Vec2::ZERO);`, then
    `get_mut(uid).formula.load_state(&json!({"rows": [src]}))`.
  - Add `plot` to the help line.
- [ ] **Tests** (extend `commands_place_and_edit` or a new `plot_command_places_an_expression`):
  - `plot sin(x)` → one viewport, tool `expression`, and `save_state()["rows"] == ["sin(x)"]`.
  - `plot r = 1 + cos(3θ)` works.
  - `plot sin(` → false, no viewport, and the log names the error.
  - `plot` → usage.
  - Undo removes the viewport and redo restores it with the expression.
- [ ] **Docs:**
  - README: a CURVES row for the Expression plotter, plus `plot` in the commands list.
  - CLAUDE.md: a short "Expressions" paragraph covering `src/expr/` (single-letter rule,
    `to_math`/`to_code`), the `export_code` hook, and the state hooks that `plot` uses now and
    Scope 6 extends.

Commit: `Add the plot command and document the expression sandbox`.

---

## Verification (before the PR)

1. Gate → `GATE OK`; `cargo check --lib --target wasm32-unknown-unknown`.
2. `check-export.sh expression` checks the static templates (ASCII, Python headless with 60
   frames, `tsc --strict`, macroquad `cargo check`).
3. `EXPR_EXPORT_DIR=… cargo test write_sample_exports -- --ignored`, then run the same three
   checks on those generated files (rows with every Func, Greek names, subscripts, and all
   definition kinds).
4. Web build plus headless Chromium (scratchpad Playwright):
   - open `#expression` and screenshot both themes;
   - type `y = tan(x)` into a row and confirm there are no vertical asymptote lines;
   - type `sin(` and confirm the error text appears;
   - use the command line: `plot r = 1 + cos(3θ)`;
   - confirm there are no page errors.
5. `cargo run --release` smoke test: place it via the toolbar, the library and `EXP`; resize to
   the minimum; pan and zoom.
6. Final whole-branch review (most capable model), then push, then open the PR (the user reviews
   and merges).
