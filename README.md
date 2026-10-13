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
| Geometry & Spatial Analysis | Euclidean distance `d = √((x2 − x1)² + (y2 − y1)²)` | nearest-point tooltip under the mouse; K-means clustering mode |
| | 2D rotation `x' = x cos θ − y sin θ, y' = x sin θ + y cos θ` | drag the point; tilt crowded axis labels until they stop colliding |
| Statistics & Trend Analysis | Line of best fit (least squares) | drag, add and remove points; residuals and squared errors; R² |
| Animation & Easing | Cubic ease-out `f(t) = 1 − (1 − t)³` | curve and speed plot, linear vs eased race, bars gliding into place |
| Color & Accessibility | Relative luminance `Y = 0.2126R + 0.7152G + 0.0722B` | heatmap picks black or white text per cell; contrast ratios; WCAG linearization toggle |
| Sorting | Bubble, Selection, Insertion, Merge, Quick, Quick (3-way), Heap and Counting sort | one shared, randomizable list (size, random / nearly sorted / reversed / few unique); play, pause, step, step back and scrub through every recorded comparison, swap and write; four skins; sound |
| | Sorting race | pick algorithms and race them on the same list with live comparison, move and timing bars |

Each page shows the formula, a live visualization, and a "Where it's used" note.
In the browser, `#id` in the URL (e.g. `index.html#mandelbrot`) opens a formula directly.

## Run

Native:

```sh
cargo run --release
# with sound on the sorting pages (Linux needs the ALSA headers: libasound2-dev)
cargo run --release --features native-audio
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
  audio.rs           tones: Web Audio in the browser, cpal on desktop (feature native-audio)
  formulas/
    mod.rs           registry, category names, glyph-coverage test
    polar.rs  projection.rs  lerp.rs  bezier.rs  lissajous.rs
    force.rs  normalize.rs  mandelbrot.rs  distance.rs  rotation.rs
    regression.rs  easing.rs  luminance.rs
    sorting/         algorithms.rs (records every step, UI-free), data.rs (shared list),
                     skin.rs (bars / radial / spectrum / dots), race.rs, mod.rs (player and view)
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
every module here does. egui's bundled font lacks some symbols (′ ∈ → ᵢ ᵣ, combining accents like x̄); `cargo test` fails if
a formula's text uses a character with no glyph.

## Sorting visualizer

Each algorithm runs once on the shared list inside a `Recorder` and logs every operation
(`Compare`, `Swap`, `Write`, `Sorted`, pivot/minimum/key markers, recursion `Enter`/`Exit`, and
counting sort's tallies). The page replays that log at the chosen speed, and seeking backwards
replays from the start, so you can scrub through a run like a video. Bars use one color language:
blue unsorted, red comparing, amber swapping or writing, green in final place, plus purple for
selection's minimum or insertion's key and pink for quick sort's pivot. Heap sort draws the heap as a
tree for lists of up to 63 values; counting sort shows its tally histogram and output array.

Extras on every sorting page:

- **Skins**: bars, radial (a clock face; swaps sweep a curve across it), color spectrum (value as hue,
  so an unsorted list is a jumbled rainbow that settles into a gradient) and dot scatter.
- **Sound**: each step plays the touched value as a pitch, `frequency = 150 + X_norm · 650` Hz, and a
  finished run ends with a rising sweep. The browser build uses the Web Audio API; the desktop build
  uses `cpal` when built with `--features native-audio`.
- **Keys**: Space plays or pauses, the Left and Right arrows step back and forward.
- **Presets**: Nearly sorted has just two or three values out of place, which is where insertion sort
  shines; Few unique is where 3-way quick sort leaves the 2-way version behind.

The **Sorting race** page runs the chosen algorithms on the same list at the same steps per second, so
fewer operations means an earlier finish. Each lane shows comparisons and swaps + writes live, and,
once it finishes, a measured time from actually running that algorithm (without recording) on this
machine.

To add another algorithm, add a variant to `Algorithm`, write it against `Recorder`, and add its
text to `info()` in `sorting/mod.rs`; the tests check every algorithm sorts correctly and leaves no
state behind.

## Credits

Silhouettes adapted from [game-icons.net](https://game-icons.net) by Lorc, Delapouite and contributors,
licensed [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/). Outlines were traced and simplified.
