# Formula

A Rust app for visualizing formulas, built on [egui](https://github.com/emilk/egui)/eframe.
It runs as a native desktop app and in the browser (WebAssembly).

The app is a CAD-style **formula workbench**. Every formula is a tool: pick it from the toolbar,
the tool library or the command line, then click the drafting canvas to place it as a live viewport
(or drag out a rectangle to choose its size). Viewports can be moved, resized, duplicated, locked,
hidden and deleted, with undo. For any formula, the Inspector shows its properties and controls,
its full reference documentation, and working visualization code in **Python**, **TypeScript** and
**Rust** to copy or download. Its equations export as **LaTeX** with the current parameter values, or
as a ready-to-run **Jupyter notebook**.

Formulas included, grouped as in the tool library:

| Category | Formula | Interaction |
| --- | --- | --- |
| Coordinate Systems & Projections | Polar to Cartesian `x = r cos θ, y = r sin θ` | drag the point; rose curve `r = cos kθ` mode |
| | Perspective projection `x' = f·x/z + cx` | drag to orbit a 3D scatter plot; dolly zoom |
| Curves & Interpolation | Linear interpolation `v(t) = v0 + t(v1 − v0)` | drag endpoints; numbers and colors too |
| | Cubic Bézier curve | drag control points; de Casteljau construction and Bernstein weights |
| | Drawn by circles (Fourier epicycles) | 64 animal silhouettes traced by up to 200 circles |
| | Lissajous figures `x = A sin(at + δπ), y = B sin(bt)` | amplitude, frequency and phase sliders; δ drifts so the figure morphs |
| | Expression plotter: type `y = a sin(bx)`, `r = 1 + cos(3θ)` or `(cos t, sin t)` | up to six rows, a slider for every free letter, breaks at asymptotes, drag to pan and scroll to zoom; the code export contains your expressions |
| Physics & Network Layouts | Springs and repulsion: Hooke `Fs = −k(d − x0)` + Coulomb `Fr = kr/d²` | drag nodes; force arrows on hover |
| Data Scaling & Normalization | Min-max normalization | outlier slider, per-bar calculation on hover |
| Fractals & Generative Patterns | Mandelbrot set `z(n+1) = z(n)² + c` | click/right-click to zoom, drag to pan, orbit on hover |
| | Perlin noise flow field `θ = N(x, y, t) · 2π · k` | particles with fading trails; field readout on hover; vector overlay |
| | Conway's Game of Life | draw/erase cells, classic patterns, the rule for the hovered cell, fading trails of dead cells |
| Chaos & Dynamical Systems | Lorenz attractor `dx/dt = σ(y − x), dy/dt = x(ρ − z) − y, dz/dt = xy − βz` | RK4 integration, drag to orbit, σ/ρ/β sliders, up to three nearly identical starts with a log plot of their separation |
| Geometry & Spatial Analysis | Euclidean distance `d = √((x2 − x1)² + (y2 − y1)²)` | nearest-point tooltip under the mouse; K-means clustering mode |
| | 2D rotation `x' = x cos θ − y sin θ, y' = x sin θ + y cos θ` | drag the point; tilt crowded axis labels until they stop colliding |
| | Voronoi diagram `V(pᵢ) = {x : d(x, pᵢ) ≤ d(x, pⱼ) for all j ≠ i}` | drag, add and remove seeds; drift; Lloyd relaxation; Delaunay edges; giraffe, cracked mud and dragonfly-wing styles |
| Linear Algebra | Matrix transformation `x' = a x + b y, y' = c x + d y` | drag the basis tips; the grid warps; det shading; eigenvectors with their λ; presets morph from the identity; chain up to three matrices |
| Statistics & Trend Analysis | Line of best fit (least squares) | drag, add and remove points; residuals and squared errors; R² |
| Animation & Easing | Cubic ease-out `f(t) = 1 − (1 − t)³` | curve and speed plot, linear vs eased race, bars gliding into place |
| Color & Accessibility | Relative luminance `Y = 0.2126R + 0.7152G + 0.0722B` | heatmap picks black or white text per cell; contrast ratios; WCAG linearization toggle |
| Sorting | Bubble, Selection, Insertion, Merge, Quick, Quick (3-way), Heap and Counting sort | one shared, randomizable list (size, random / nearly sorted / reversed / few unique); play, pause, step, step back and scrub through every recorded comparison, swap and write; four skins; sound |
| | Sorting race | pick algorithms and race them on the same list with live comparison, move and timing bars |
| Graphs & Trees | Breadth-first, depth-first, Dijkstra and A* search `dist(n) = dist(u) + 1` | draw walls and mud on a shared grid, drag the start and goal, mazes and random walls; play, step and scrub through every frontier, visit and path step |
| | Binary search tree (AVL: balance factor within 1) | insert, delete and find a key; random trees; toggle AVL balancing; in-, pre- and post-order walks; every comparison and rotation is a frame you can play, step and scrub, and rotations glide the nodes into place |
| | Kruskal's and Prim's minimum spanning trees `minimize Σ w(e) over the n − 1 tree edges` | one shared graph of 6 to 24 random points with Delaunay edges; drag points to move them; edges turn considering, accepted or rejected as the run replays; Kruskal colors vertices by tree; both list the edges still waiting by weight; play, step and scrub |

In the browser, `#id` in the URL (e.g. `index.html#mandelbrot`) brings that formula into view,
placing it first if needed.

## The workbench

| Area | What it does |
| --- | --- |
| Toolbar | Select (V) and Pan (H), undo/redo, zoom; one button per category with a menu of its tools |
| Browser (left) | Tool library (click to place, double-click to place beside the drawing) and the list of viewports with show/hide and lock |
| Canvas | Infinite drafting plane with an adaptive grid, origin axes, rulers, crosshair, navigator (with zoom, fit and 1:1 buttons) and title block |
| Inspector (right) | **Properties**: name, X/Y/W/H, lock, the equations and the formula's controls. **Docs**: the full reference. **Code**: Python, TypeScript and Rust, with Copy and Download (Save on desktop), plus **LaTeX** (the equations with the viewport's current values) and a Jupyter **Notebook** (`.ipynb`). The maximize button widens it for reading |
| Command line | Type a tool alias (`LOR`, `BEZ`, `MAN`, `VOR`, …) or name to place it, or a command: `fit`, `zoom 150`, `del`, `dup`, `undo`, `redo`, `grid`, `snap`, `ortho`, `latex` (copy the selection's equations as LaTeX), `ipynb` (export it as a notebook), `plot <expression>` (place an expression plotter, e.g. `plot r = 1 + cos(3θ)`), `help` |
| Status bar | Pointer coordinates, current tool, SNAP / GRID / ORTHO toggles, zoom, viewport count, fps |

Working with viewports:

- **Select** a viewport by clicking it or its title tab; Shift adds to the selection. Drag on empty
  canvas for a window selection (left to right: fully inside) or crossing selection (right to left:
  touching).
- **Use** the selected (active) viewport: dragging inside it goes to the formula, so you can move
  Bézier control points, draw Life cells or orbit the Lorenz attractor.
- **Move** by dragging the title tab, or an inactive viewport. Moves snap to the grid and to the
  edges and centers of other viewports, with dashed guides; ORTHO (or Shift) keeps a move on one axis.
- **Resize** with the eight grips; Shift keeps the aspect ratio. A viewport cannot shrink below the
  size its formula needs. The selected viewport shows its width and height as dimension lines.
- **Navigate** with the wheel (zoom about the pointer), pinch, middle-drag or Shift+wheel (pan),
  F (fit all), Shift+F (fit selection), Ctrl+0 (100%), or by clicking the navigator.
- **Edit** with Ctrl+Z / Ctrl+Y, Ctrl+D (duplicate), Del, arrow keys (nudge; Shift for ×10), and
  right-click menus on viewports and the canvas.

Formulas are drawn at their natural size (one world unit is one point) on a layer that egui scales
and translates as a whole, so every formula keeps its own layout at any zoom, and lines stay crisp.
The layout, camera, theme and settings are saved between sessions.

## Run

Native:

```sh
cargo run --release
# with sound on the sorting viewports (Linux needs the ALSA headers: libasound2-dev)
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

The exported code in `assets/content/<id>/` runs on its own:

```sh
pip install numpy matplotlib && python main.py              # Python
tsc main.ts                                                 # TypeScript: load main.js in any page
cargo new demo && cd demo && cargo add macroquad@0.4        # Rust: replace src/main.rs, then
cargo run --release
```

## Layout

```text
src/
  main.rs            native entry point
  web.rs             browser entry point and #id deep links
  app.rs             shell: wires the panels and the canvas together; saves the workspace
  content.rs         embedded reference docs and code (table generated by build.rs)
  cad/               the workbench
    canvas.rs        layers, input routing, placing / selecting / moving / resizing, commands
    camera.rs        pan and zoom with smoothing, fit, grid steps (UI-free)
    board.rs         viewports, z-order, selection, smart guides, saved layout (UI-free)
    history.rs       undo / redo
    overlay.rs       grid, rulers, frames, grips, dimensions, guides, crosshair, navigator
    chrome.rs        menu bar, toolbar, Browser, command line, status bar, help
    inspector.rs     Properties, Docs and Code tabs
    markdown.rs      the docs renderer      highlight.rs  syntax highlighting
    icons.rs         line-art tool and UI icons   theme.rs  graphite and drafting-paper themes
    export.rs        copy, download (web) and save (desktop)
  formula.rs         the `Formula` trait, `Palette`, `MathText` equation builder
  draw.rs            drawing helpers: polygon fill, lerp/gradients, draggable handles
  rng.rs             small seeded random number generator
  expr/              expression engine (UI-free): lexer, Pratt parser, eval, math and code printers
  audio.rs           tones: Web Audio in the browser, cpal on desktop (feature native-audio)
  formulas/
    mod.rs           the tool catalogue (`TOOLS`), category names, glyph-coverage test
    polar.rs  projection.rs  lerp.rs  bezier.rs  lissajous.rs
    force.rs  normalize.rs  mandelbrot.rs  flowfield.rs  life.rs  lorenz.rs
    distance.rs  rotation.rs  voronoi.rs  regression.rs  easing.rs  luminance.rs
    sorting/         algorithms.rs (records every step, UI-free), data.rs (shared list),
                     skin.rs (bars / radial / spectrum / dots), race.rs, mod.rs (player and view)
    fourier/         epicycles: math.rs (UI-free), animals.rs, mod.rs (view)
    expression/      plotter: plot.rs (view and sampling, UI-free), export.rs (code generation), mod.rs
assets/animals.json  animal outlines
assets/content/<id>/ doc.md, main.py, main.ts, main.rs for every tool
build.rs             embeds assets/content
```

## Adding a formula

1. Create a module under `src/formulas/` and implement `formula::Formula` for its state.
   Required: `id`, `name`, `category`, `summary`, `equations`, `paint` (draws the stage every
   frame) and `controls`.
   Optional: `notes` ("Where it's used"), `details`, `stage_height`, `is_animating`, and
   `stage_sense` + `interact` for dragging or clicking on the stage.
   Optional: `min_size` (the smallest viewport it lays out in), `handle_keys` (keyboard shortcuts while
   it is the active viewport) and `wants_scroll` (keep the mouse wheel for itself).
2. Add a `tool!` line to `TOOLS` in `formulas/mod.rs` with its id, name, category and a three-letter
   command alias. Use one of the category constants, or add a new one; the toolbar and library group
   tools by category automatically.
3. Add `assets/content/<id>/` with `doc.md` (the ten sections listed in `content::SECTIONS`),
   `main.py`, `main.ts` and `main.rs`. `cargo test` checks that every tool has all four.

The workbench takes care of placing, selecting, moving and resizing viewports, the panels, theming,
undo and saving the layout. Each viewport has its own instance of the formula. Use the `Palette` colors (`x`, `y`, `trace`, `fill`, …) so new formulas match the
existing ones in both light and dark mode. Keep the math in plain functions with unit tests, as
every module here does. egui's bundled font lacks some symbols (′ ∈ → ᵢ ᵣ, combining accents like x̄); `cargo test` fails if
a formula's text uses a character with no glyph.

## Sorting visualizer

Sorting viewports share one list per sheet, so a shuffle in one shows up in every other and
algorithms can be compared on the same input. Each algorithm runs once on the shared list inside a `Recorder` and logs every operation
(`Compare`, `Swap`, `Write`, `Sorted`, pivot/minimum/key markers, recursion `Enter`/`Exit`, and
counting sort's tallies). The viewport replays that log at the chosen speed, and seeking backwards
replays from the start, so you can scrub through a run like a video. Bars use one color language:
blue unsorted, red comparing, amber swapping or writing, green in final place, plus purple for
selection's minimum or insertion's key and pink for quick sort's pivot. Heap sort draws the heap as a
tree for lists of up to 63 values; counting sort shows its tally histogram and output array.

Extras on every sorting viewport:

- **Skins**: bars, radial (a clock face; swaps sweep a curve across it), color spectrum (value as hue,
  so an unsorted list is a jumbled rainbow that settles into a gradient) and dot scatter.
- **Sound**: each step plays the touched value as a pitch, `frequency = 150 + X_norm · 650` Hz, and a
  finished run ends with a rising sweep. The browser build uses the Web Audio API; the desktop build
  uses `cpal` when built with `--features native-audio`.
- **Keys**: while a sorting viewport is selected, Space plays or pauses and the Left and Right arrows
  step back and forward.
- **Presets**: Nearly sorted has just two or three values out of place, which is where insertion sort
  shines; Few unique is where 3-way quick sort leaves the 2-way version behind.

The **Sorting race** runs the chosen algorithms on the same list at the same steps per second, so
fewer operations means an earlier finish. Each lane shows comparisons and swaps + writes live, and,
once it finishes, a measured time from actually running that algorithm (without recording) on this
machine.

To add another algorithm, add a variant to `Algorithm`, write it against `Recorder`, and add its
text to `info()` in `sorting/mod.rs`; the tests check every algorithm sorts correctly and leaves no
state behind.

## Credits

Silhouettes adapted from [game-icons.net](https://game-icons.net) by Lorc, Delapouite and contributors,
licensed [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/). Outlines were traced and simplified.
