# Formula

A Rust app for visualizing formulas, built on [egui](https://github.com/emilk/egui)/eframe.
It runs as a native desktop app and in the browser (WebAssembly).

Formulas included:

- **Drawn by circles**: Fourier epicycles trace 64 animal silhouettes using up to 200 spinning circles.
  Lower the number of terms `N` and watch the detail melt away.

  ```text
  x_N(t) = Σ_{k=1..N} (a_k cos kt + b_k sin kt)
  y_N(t) = Σ_{k=1..N} (c_k cos kt + d_k sin kt)
  ```

- **Lissajous figures**: `x(t) = sin(a t + δπ)`, `y(t) = sin(b t)`.

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
  web.rs             browser entry point
  app.rs             shell: formula catalogue, page layout, theme
  formula.rs         the `Formula` trait, `Palette`, `MathText` equation builder
  draw.rs            drawing helpers (non-convex polygon fill)
  formulas/
    mod.rs           registry of all formulas
    lissajous.rs
    fourier/
      mod.rs         the epicycle visualization and animal picker
      math.rs        resampling, normalization, Fourier series (no UI; unit-tested)
      animals.rs     bundled silhouettes
assets/animals.json  animal outlines
```

## Adding a formula

1. Create a module under `src/formulas/` and implement `formula::Formula` for its state:
   `id`, `name`, `summary`, `equations`, `paint` (draws the stage every frame) and `controls`.
   `details`, `stage_height` and `is_animating` are optional.
2. Add it to `formulas::registry()`.

The shell takes care of the sidebar, title, stage frame, equations, theming and persistence of the
selected formula. Use the `Palette` colors (`x`, `y`, `trace`, `fill`, …) so new formulas match the
existing ones in both light and dark mode. Keep the math in a separate UI-free module so it can be
unit-tested, as `fourier/math.rs` is.

## Credits

Silhouettes adapted from [game-icons.net](https://game-icons.net) by Lorc, Delapouite and contributors,
licensed [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/). Outlines were traced and simplified.
