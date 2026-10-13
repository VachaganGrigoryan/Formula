# Formula

A Rust app for visualizing formulas, built on [egui](https://github.com/emilk/egui)/eframe.
It runs as a native desktop app and in the browser (WebAssembly).

Formulas included, grouped as in the sidebar:

| Category | Formula | Interaction |
| --- | --- | --- |
| Coordinate Systems & Projections | Polar to Cartesian `x = r cos θ, y = r sin θ` | drag the point; rose curve `r = cos kθ` mode |
| | Perspective projection `x' = f·x/z + cx` | drag to orbit a 3D scatter plot; dolly zoom |
| Curves & Interpolation | Linear interpolation `v(t) = v0 + t(v1 − v0)` | drag endpoints; numbers and colors too |
| | Cubic Bézier curve | drag control points; de Casteljau construction and Bernstein weights |
| | Drawn by circles (Fourier epicycles) | 64 animal silhouettes traced by up to 200 circles |
| | Lissajous figures `x = sin(at + δπ), y = sin(bt)` | frequency and phase sliders |
| Physics & Network Layouts | Springs and repulsion: Hooke `Fs = −k(d − x0)` + Coulomb `Fr = kr/d²` | drag nodes; force arrows on hover |
| Data Scaling & Normalization | Min-max normalization | outlier slider, per-bar calculation on hover |
| Fractals & Generative Patterns | Mandelbrot set `z(n+1) = z(n)² + c` | click/right-click to zoom, drag to pan, orbit on hover |

Each page shows the formula, a live visualization, and a "Where it's used" note.
In the browser, `#id` in the URL (e.g. `index.html#mandelbrot`) opens a formula directly.

## Run

Native:

```sh
cargo run --release
```

On Linux you need the usual windowing libraries (`libxkbcommon-x11`, an OpenGL driver).

Web:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129   # must match the wasm-bindgen version in Cargo.lock
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/release/formula.wasm
python3 -m http.server   # then open http://localhost:8000
```

Tests:

```sh
cargo test
```

## Layout

```text
src/
  main.rs            native entry point
  web.rs             browser entry point and #id deep links
  app.rs             shell: grouped catalogue, page layout, theme
  formula.rs         the `Formula` trait, `Palette`, `MathText` equation builder
  draw.rs            drawing helpers: polygon fill, lerp/gradients, draggable handles
  rng.rs             small seeded random number generator
  formulas/
    mod.rs           registry, category names, glyph-coverage test
    polar.rs  projection.rs  lerp.rs  bezier.rs  lissajous.rs
    force.rs  normalize.rs  mandelbrot.rs
    fourier/         epicycles: math.rs (UI-free), animals.rs, mod.rs (view)
assets/animals.json  animal outlines
```

## Adding a formula

1. Create a module under `src/formulas/` and implement `formula::Formula` for its state.
   Required: `id`, `name`, `category`, `summary`, `equations`, `paint` (draws the stage every
   frame) and `controls`.
   Optional: `notes` ("Where it's used"), `details`, `stage_height`, `is_animating`, and
   `stage_sense` + `interact` for dragging or clicking on the stage.
2. Add it to `formulas::registry()`. Use one of the category constants in `formulas/mod.rs`, or
   add a new one; the sidebar groups formulas by category automatically.

The shell takes care of the sidebar, title, stage frame, equations, theming and remembering the
selected formula. Use the `Palette` colors (`x`, `y`, `trace`, `fill`, …) so new formulas match the
existing ones in both light and dark mode. Keep the math in plain functions with unit tests, as
every module here does. egui's bundled font lacks some symbols (′ ∈ → ᵣ); `cargo test` fails if
a formula's text uses a character with no glyph.

## Credits

Silhouettes adapted from [game-icons.net](https://game-icons.net) by Lorc, Delapouite and contributors,
licensed [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/). Outlines were traced and simplified.
