# Scope 1: Step-by-step derivations and proofs (roadmap phases 1.1–1.4)

> Execution: inline in the main thread (user preference, 2026-10-18). TDD per task, the gate
> before every commit, one PR at the end for the user to review.

## Context

Scopes 0, 5, 4, 2 and 3 are merged (main `1324d25`). Scope 1 is next in the approved order
(…→3→**1**→6). Today the app shows a formula's equations but not *why* they hold, or how the
current numbers flow through them. Scope 1 adds:
- a **Derive** tab, which reveals a derivation one step at a time;
- a generic **"Plug in current values"** derivation, built on the Scope 3 expression engine;
- authored derivations for five existing tools;
- a new **Proofs & Derivations** category with three tools.

**Branch:** `feat/derivations` from `main` `1324d25`.

## Rulings made while planning

- **The plug-in hook returns a list of substitutions.** Its signature is
  `fn substitutions(&self) -> Vec<Substitution>`, not the roadmap's
  `fn expr(&self) -> Option<(Expr, Vec<(String, f64)>)>`. Polar and rotation have two equations
  (x and y), so one expression is not enough. If this is wrong, it costs a rename.
- **Tools parse their own source strings.** Each tool builds its expression with
  `expr::parse_expr("sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)")` and does not construct trees by hand.
  This reuses the tested parser, and a test asserts that every tool's source parses.
- **Plug-in steps show numbers to at most 4 significant digits** (`latex::number`). Evaluation stays
  exact, and the invariant test uses exact values.
- **Negating a literal is not a step of its own.** `Neg(Num)` folds into the step that created it,
  so no step looks identical to the one before. The step count is therefore 1 plus the number of
  `Bin`/`Call` nodes plus the number of `Neg` nodes over a non-literal.
- **A `derive` command** opens the tab, like `docs` and `code` do. It is small and consistent with
  those commands.
- **Proof tools go last** in `TOOLS` (category PROOFS, toolbar name "Proofs").

## Global Constraints

- The gate passes before every commit; wasm builds; `board-v1` saves still load; Palette colors
  only; math in UI-free functions with tests; no new dependencies.
- Every new tool meets the full checklist:
  - a `TOOLS` entry with a unique alias;
  - an icon arm;
  - 4 content files, with the 10 doc sections and ASCII-only code;
  - glyphs exist for all text;
  - `paint` survives `min_size`;
  - controls leave `params()` unchanged;
  - every equation converts to ASCII LaTeX;
  - the notebook and `plt.show()` tests pass.
- Glyph rule: every derivation step and `why` must render in egui's font. Use `·` not `⋅`,
  `->`/words not `→`, and no `∈`, `′`, `ᵢ`, or combining accents.

## Review Focus

1. **Plug-in on degenerate values**: `x_max == x_min` (divide by zero), `t` at 0 or 1, `θ = 0`.
   Steps must still be produced, end in a number (∞ or NaN is shown as text), and never panic.
   [Task 2]
2. **The Derive tab with no selection, with a tool without derivations, or when the selection
   changes while step 5 of 7 is shown.** Expected: a hint; the step index resets or clamps, with
   no out-of-bounds access. [Task 1]
3. **Old saved `tab` values** (`"Properties"`, `"Docs"`, `"Code"`) still deserialize after adding
   `Derive`. [Task 1]
4. **Quadratic with a = 0 or a negative discriminant.** Expected: a caption ("linear: one root" /
   "no real roots"), with no NaN plotted and no NaN root markers. [Task 5]
5. **Euler partial sums at large θ** (θ up to 2π, n up to 30) converge, and the arrows stay inside
   the stage at `min_size`. [Task 7]

---

## File map

| File | Role |
|---|---|
| `src/derive.rs` (new) | `Step`, `Derivation`, `Substitution`, `substitution_steps`, `plug_in` (Derivation from substitutions) |
| `src/expr/print.rs` | `pub fn expr_math(e, focus: Option<&[usize]>) -> MathText`: one expression, the focused subtree tinted, numbers to 4 significant digits |
| `src/formula.rs` | hooks `derivations()` and `substitutions()` |
| `src/cad/inspector.rs` | `Tab::Derive`, its view, the step state |
| `src/cad/canvas.rs` | `Request::Derive`, the `derive` command |
| `src/cad/icons.rs` | `Ui::Derive` icon, plus icon arms for the 3 new tools |
| `src/formulas/{distance,lerp,polar,rotation,luminance,normalize,easing}.rs` | `substitutions()` |
| `src/formulas/{distance,rotation,regression,polar,lerp}.rs` | authored `derivations()` |
| `src/formulas/proofs/{mod,quadratic,pythagoras,euler}.rs` (new) | the three proof tools |
| `src/formulas/mod.rs`, `src/cad/chrome.rs` | the PROOFS category, registration, the glyph test extended |
| `assets/content/{quadratic-formula,pythagorean-proof,euler-identity}/*` | docs and exports |

---

### Task 1: Derivation model and the Derive tab (phase 1.1)

**`src/derive.rs`:**
```rust
pub struct Step { pub math: MathText, pub why: String }
pub struct Derivation { pub title: String, pub steps: Vec<Step> }
```
`why` is a `String`, so the plug-in steps can say "multiply 2 · 3".

**Hook** in `formula.rs`:
```rust
fn derivations(&self) -> Vec<Derivation> { Vec::new() }
```
The Inspector shows `plug_in(self.substitutions())` (Task 2) first when it is non-empty, then
`derivations()`.

**Inspector:**
- `Tab::Derive`, appended after `Code` in the enum. The tab buttons read
  Properties, Docs, Derive, Code.
- State: `derive_pick: usize` (which derivation) and `derive_step: usize` (steps revealed), both
  keyed by the context tool id. When the tool id changes, both reset to 0. Before use, both are
  clamped to the current lengths.
- View:
  - a picker when there are several derivations (selectable labels with the titles);
  - every step up to `derive_step`: its math at size 15, then its `why` in `chrome.text_dim`;
  - the newest step framed in `chrome.well`;
  - buttons: Prev / Next (also Left/Right while the tab is hovered), "Show all", "Restart";
  - a counter "step k of n".
  - With no tool, the hint "Select a viewport to see how its formula is derived." With no
    derivations, "No derivation for this tool yet."
- `Request::Derive`, plus `"derive"` in `run_command` and the help line.

**`Ui::Derive` icon:** three short stacked lines with an `=` at the left.

**Tests:**
- `Tab` serde: `serde_json::from_str::<Tab>("\"Code\"")` is `Code`, and `"Derive"` round-trips.
- A pure helper `fn clamp_steps(pick, step, lens: &[usize]) -> (usize, usize)`: an out-of-range
  pick or step clamps, and empty `lens` gives (0, 0) (Review Focus 2).
- The `derive` command sets `ws.request == Some(Request::Derive)`.
- `all_text_has_glyphs` is extended: for every tool, the title, math text and `why` of every
  `derivations()` step and of `plug_in(substitutions())`.

Commit: `Add a Derive tab that reveals derivations step by step`.

### Task 2: Substitution steps (phase 1.2)

```rust
pub struct Substitution {
    pub lhs: MathText,                 // e.g. "d" toned Trace
    pub expr: Expr,                    // parsed from the tool's source string
    pub values: Vec<(String, f64)>,    // variable name -> current value
}
pub struct Reduction { pub expr: Expr, pub focus: Option<Vec<usize>>, pub why: String }
pub fn substitution_steps(e: &Expr, values: &dyn Fn(&str) -> Option<f64>) -> Vec<Reduction>;
pub fn plug_in(subs: &[Substitution]) -> Option<Derivation>;   // None when subs is empty
```

**Hook:** `fn substitutions(&self) -> Vec<Substitution> { Vec::new() }`.

**`substitution_steps`:**
1. The first step replaces every variable that has a value with `Num(v)` (a negative value
   becomes `Neg(Num(|v|))`, folded). Constants π and e stay symbolic until reduced. `why` is
   "substitute x_1 = 3, ...".
2. Each later step reduces the **innermost-leftmost** `Bin`/`Call`/`Neg` whose children are all
   `Num` to `Num(eval)`. `focus` is its path (the child indices from the root). `why` is
   "subtract", "multiply", "square root", ... (one word per `BinOp`/`Func`). A `Neg` over a literal
   folds immediately.
3. It stops when a single `Num` remains.

**`expr_math(e, focus)`:**
- the same layout rules as `to_math` (powers as superscripts, subscripted names);
- numbers via `latex::number`;
- negative literals in parentheses when they are the right operand or a power base;
- the focus subtree toned `Tone::Trace`.

`to_math` reuses it.

**`plug_in`:**
- The derivation is titled "Plug in current values".
- For each substitution: one step with the symbolic equation (`lhs = expr`, "the formula"), then
  one step per `Reduction`, each shown as `lhs = …`.

**Tests (`derive.rs`):**
- Every step evaluates to the same number as the first, within 1e-9 relative (the invariant), for
  50 seeded random expressions (`expr::random_expr`) with random positive values.
- The last step is a `Num`.
- The step count matches the ruled formula.
- `sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)` with (0,0),(3,4) ends at 5, and the step `why`s are
  `[substitute, subtract, square, subtract, square, add, square root]`. Squaring a power is
  written "power", and "square" is used when the exponent is 2.
- Degenerate values (Review Focus 1): `(x - a)/(b - a)` with a = b ends in ∞ or NaN with no panic;
  the last step's math text is "∞" or "NaN" shown with a glyph that exists (use "undefined" for
  NaN and "∞" for infinity, after a glyph check).
- `expr_math` tints exactly the focused subtree: its Trace-toned pieces equal that subtree's text.

**Implement `substitutions()`** for 7 tools, each from a source string:

| Tool | Substitutions |
|---|---|
| distance | `d = sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)`, with the probe and its nearest point (empty if there are no points) |
| lerp | `v = v_0 + t (v_1 - v_0)` |
| polar | `x = r cos(θ)`, `y = r sin(θ)` |
| rotation | `x' = x cos(θ) - y sin(θ)`, `y' = x sin(θ) + y cos(θ)` |
| luminance | `Y = 0.2126 R + 0.7152 G + 0.0722 B` |
| normalize | `X_norm = (X - X_min) / (X_max - X_min)`, with X the hovered value or else the first |
| easing | `f = 1 - (1 - t)^3` |

`lhs` is a `MathText` (`x'` is text, not parsed). A test asserts every tool's
`substitutions()` end in a finite number at the default state, and that each source
`parse_expr`s.

Commit: `Plug the current values into a formula, one operation per step`.

### Task 3: Authored derivations (phase 1.3)

Content in `derivations()`. Each step is a `MathText` plus a sentence.
- **distance**, "From Pythagoras": the right triangle with legs Δx, Δy, then a² + b² = c², then
  substitute the legs, then take the square root.
- **rotation**, "From the angle-addition identities": a point at angle α and radius r, add θ,
  expand cos(α+θ) and sin(α+θ), and substitute x = r cos α, y = r sin α.
- **regression**, "Least squares": minimize E = Σ(y_i − (m x_i + b))², set ∂E/∂b = 0 to get
  b = μ_y − m μ_x, set ∂E/∂m = 0, substitute, and arrive at m.
- **polar**, "The unit circle": cos and sin as the coordinates of the unit-circle point, scaled by r.
- **lerp**, "A weighted average": v = (1 − t) v_0 + t v_1, regroup to v_0 + t(v_1 − v_0), with
  t = 0 and t = 1 as checks.

Use `Tone` on the parts each step introduces. Tests: the extended glyph test, and the LaTeX
conversion of every step's math is ASCII (extend `every_equation_converts_to_ascii_latex` to
derivation steps).

Commit: `Add derivations for distance, rotation, regression, polar and lerp`.

### Task 4: The PROOFS category and shared pieces

- `pub const PROOFS: &str = "Proofs & Derivations";` and `"Proofs"` in `chrome::short`.
- `src/formulas/proofs/mod.rs` with the three submodules.
- Each tool lives in its own task below and is registered at the end of `TOOLS`.

### Task 5: `quadratic-formula` (QDR)

**Math (UI-free):**
```rust
pub enum Roots { Two(f64, f64), One(f64), None, Linear(Option<f64>) }
pub fn roots(a, b, c) -> Roots
```
`Two` is returned in ascending order. `Linear` covers a = 0.

**Tool:**
- sliders for a, b, c (−5..5, `Edits` clamping);
- the parabola on axes (reuse `expression::plot::{View, sample_explicit, nice_step}`);
- root markers, the vertex, the axis of symmetry;
- a caption for the discriminant case (Review Focus 4).

**Derivations:**
- "Completing the square": 7 steps from ax² + bx + c = 0 to the formula.
- `substitutions()`: x_1 and x_2 as two plus/minus expressions (`(-b + sqrt(b^2 - 4a c))/(2a)`
  and the `-` variant), plus Δ = b² − 4ac.

**Tests:**
- Roots match `expr::eval` of both sources.
- Each root satisfies |ax² + bx + c| < 1e-9 on 200 seeded cases.
- The a = 0 and Δ < 0 cases.
- The vertex is at −b/2a.

Content: doc, plus exports in all three languages with a, b, c sliders/keys.

### Task 6: `pythagorean-proof` (PYT)

**Math:**
- `pub fn triangles(a, b, t) -> [[[f64; 2]; 3]; 4]` gives the four right triangles inside the
  (a+b)² square. Keyframe 0 is the "c² tilted square" arrangement and keyframe 1 is the "a² + b²
  two squares" arrangement. In between, each triangle moves rigidly: its rotation and translation
  are interpolated with an eased t.
- `pub fn area(tri) -> f64`.

**Tool:**
- sliders a, b; Play/Pause; a t scrubber;
- a mode picker: "Rearrangement" or "Similar triangles". The second draws the altitude to the
  hypotenuse, which splits c into p and q, then a² = c·p and b² = c·q, so a² + b² = c².
- `derivations()` gives both proofs as steps.

**Tests:**
- Each triangle's area = ab/2 at t ∈ {0, 0.25, 0.5, 0.75, 1}, and its side lengths are preserved
  (rigid).
- At t = 0 the uncovered area = c². At t = 1 it = a² + b².
- All vertices stay inside the big square.

Content: doc, plus exports.

### Task 7: `euler-identity` (EUL)

**Math:**
- `pub fn partial_sums(theta, n) -> Vec<[f64; 2]>`, the complex partial sums of Σ (iθ)^k / k!,
  computed iteratively (term_{k+1} = term_k · iθ / (k+1)).

**Tool:**
- θ slider (0..2π) and n slider (1..30);
- the unit circle, with arrows head-to-tail for each term (alternating x/y tones), the point
  e^{iθ}, and a readout;
- a "θ = π" button that shows e^{iπ} + 1 = 0.

**Derivation:** the Taylor series of e^x, cos and sin, then substitute iθ and group the real and
imaginary parts.

**Tests:**
- n = 30 is within 1e-6 of (cos θ, sin θ) for θ in 0..2π at 50 points.
- θ = π gives (−1, 0) within 1e-6.
- The partial sums at n = 0 and n = 1 are (1, 0) and (1, θ).
- Arrows fit the stage at `min_size` (Review Focus 5; scale chosen from the max |partial sum|).

Content: doc, plus exports.

### Task 8: Docs and finish

- README: a Derive tab line in the Inspector row, the `derive` command, three rows in the
  Proofs table, and the `proofs/` source layout.
- CLAUDE.md: the derivation hooks (`derivations`, `substitutions`), `derive.rs`, and the PROOFS
  category.
- Verification below, then push and open the PR.

## Verification

1. Gate → `GATE OK`; `cargo check --lib --target wasm32-unknown-unknown`.
2. `check-export.sh` for each of the three new tools.
3. Web build plus headless Chromium:
   - the Derive tab on distance (plug-in steps advance with Next), rotation (authored), quadratic
     (both derivations), Pythagoras (mid-animation) and Euler (θ = π);
   - light and dark themes;
   - no page errors.
4. Native smoke test (`cargo run --release`), left for the user, as before.
