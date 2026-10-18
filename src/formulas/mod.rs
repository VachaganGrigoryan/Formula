//! Every formula the app can visualize.
//!
//! Adding one: create a module here, implement [`Formula`] for its state,
//! add a `tool!` entry to [`TOOLS`], an icon in `cad::icons::tool`, and its
//! docs and code in `assets/content/<id>/`. The tests check all of these.

pub mod bezier;
pub mod distance;
pub mod easing;
pub mod expression;
pub mod flowfield;
pub mod force;
pub mod fourier;
pub mod graphs;
pub mod lerp;
pub mod life;
pub mod lissajous;
pub mod lorenz;
pub mod luminance;
pub mod mandelbrot;
pub mod matrix;
pub mod normalize;
pub mod polar;
pub mod projection;
pub mod regression;
pub mod replay;
pub mod rotation;
pub mod sorting;
pub mod voronoi;

use crate::formula::Formula;

// Sidebar groups. Categories appear in the order their first formula is registered.
pub const COORDINATES: &str = "Coordinate Systems & Projections";
pub const CURVES: &str = "Curves & Interpolation";
pub const PHYSICS: &str = "Physics & Network Layouts";
pub const SCALING: &str = "Data Scaling & Normalization";
pub const FRACTALS: &str = "Fractals & Generative Patterns";
pub const CHAOS: &str = "Chaos & Dynamical Systems";
pub const GEOMETRY: &str = "Geometry & Spatial Analysis";
pub const STATISTICS: &str = "Statistics & Trend Analysis";
pub const ANIMATION: &str = "Animation & Easing";
pub const COLOR: &str = "Color & Accessibility";
pub const LINEAR_ALGEBRA: &str = "Linear Algebra";
pub const SORTING: &str = "Sorting";
pub const GRAPHS: &str = "Graphs & Trees";

/// Shown on first launch, before a choice has been remembered.
pub const DEFAULT_ID: &str = "fourier-epicycles";

/// What a new formula instance may share with the others on the same board.
pub struct ToolEnv {
    /// One sorting session per board, so every sorting viewport works on the
    /// same list and shares one audio output.
    pub sorting: sorting::Session,
    /// One pathfinding grid and one MST graph per board, shared by the graph
    /// and tree tools.
    pub graphs: graphs::Session,
}

impl ToolEnv {
    pub fn new() -> Self {
        Self {
            sorting: sorting::Session::new(),
            graphs: graphs::Session::new(),
        }
    }
}

impl Default for ToolEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// A formula as a workspace tool: its catalogue entry and how to make a fresh
/// instance of it.
pub struct Tool {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    /// Short command alias, typed into the command line like a CAD command.
    pub alias: &'static str,
    pub make: fn(&ToolEnv) -> Box<dyn Formula>,
}

impl PartialEq for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn new<T: Formula + Default + 'static>(_: &ToolEnv) -> Box<dyn Formula> {
    Box::new(T::default())
}

fn sort<const A: usize>(env: &ToolEnv) -> Box<dyn Formula> {
    let alg = sorting::algorithms::Algorithm::ALL[A];
    Box::new(sorting::SortVisualizer::new(alg, env.sorting.clone()))
}

fn path<const S: usize>(env: &ToolEnv) -> Box<dyn Formula> {
    Box::new(graphs::pathfinding::Pathfinding::new(
        graphs::path::Search::ALL[S],
        env.graphs.clone(),
    ))
}

fn mst<const S: usize>(env: &ToolEnv) -> Box<dyn Formula> {
    Box::new(graphs::spanning::SpanningTree::new(
        graphs::mst::Spanning::ALL[S],
        env.graphs.clone(),
    ))
}

fn race(env: &ToolEnv) -> Box<dyn Formula> {
    Box::new(sorting::race::Race::new(env.sorting.clone()))
}

macro_rules! tool {
    ($id:literal, $name:literal, $cat:ident, $alias:literal, $make:expr) => {
        Tool {
            id: $id,
            name: $name,
            category: $cat,
            alias: $alias,
            make: $make,
        }
    };
}

/// The catalogue, in the order shown in the tool library.
pub static TOOLS: &[Tool] = &[
    tool!(
        "polar-to-cartesian",
        "Polar to Cartesian",
        COORDINATES,
        "POL",
        new::<polar::Polar>
    ),
    tool!(
        "perspective-projection",
        "Perspective projection",
        COORDINATES,
        "PRJ",
        new::<projection::Projection>
    ),
    tool!(
        "lerp",
        "Linear interpolation",
        CURVES,
        "LRP",
        new::<lerp::Lerp>
    ),
    tool!(
        "cubic-bezier",
        "Cubic Bézier curve",
        CURVES,
        "BEZ",
        new::<bezier::Bezier>
    ),
    tool!(
        "fourier-epicycles",
        "Drawn by circles",
        CURVES,
        "FOU",
        new::<fourier::Epicycles>
    ),
    tool!(
        "lissajous",
        "Lissajous figures",
        CURVES,
        "LIS",
        new::<lissajous::Lissajous>
    ),
    tool!(
        "expression",
        "Expression plotter",
        CURVES,
        "EXP",
        new::<expression::Expression>
    ),
    tool!(
        "force-layout",
        "Springs and repulsion",
        PHYSICS,
        "FRC",
        new::<force::ForceLayout>
    ),
    tool!(
        "min-max-normalization",
        "Min-max normalization",
        SCALING,
        "NRM",
        new::<normalize::Normalize>
    ),
    tool!(
        "mandelbrot",
        "Mandelbrot set",
        FRACTALS,
        "MAN",
        new::<mandelbrot::Mandelbrot>
    ),
    tool!(
        "flow-field",
        "Perlin noise flow field",
        FRACTALS,
        "FLW",
        new::<flowfield::FlowField>
    ),
    tool!(
        "game-of-life",
        "Conway's Game of Life",
        FRACTALS,
        "LIF",
        new::<life::Life>
    ),
    tool!(
        "lorenz",
        "Lorenz attractor",
        CHAOS,
        "LOR",
        new::<lorenz::Lorenz>
    ),
    tool!(
        "euclidean-distance",
        "Euclidean distance",
        GEOMETRY,
        "DST",
        new::<distance::Distance>
    ),
    tool!(
        "rotation-matrix",
        "2D rotation",
        GEOMETRY,
        "ROT",
        new::<rotation::Rotation>
    ),
    tool!(
        "voronoi",
        "Voronoi diagram",
        GEOMETRY,
        "VOR",
        new::<voronoi::Voronoi>
    ),
    tool!(
        "matrix-transform",
        "Matrix transformation",
        LINEAR_ALGEBRA,
        "MAT",
        new::<matrix::MatrixTransform>
    ),
    tool!(
        "linear-regression",
        "Line of best fit",
        STATISTICS,
        "REG",
        new::<regression::Regression>
    ),
    tool!(
        "ease-out-cubic",
        "Cubic ease-out",
        ANIMATION,
        "EAS",
        new::<easing::Easing>
    ),
    tool!(
        "relative-luminance",
        "Relative luminance",
        COLOR,
        "LUM",
        new::<luminance::Luminance>
    ),
    tool!("bubble-sort", "Bubble sort", SORTING, "BUB", sort::<0>),
    tool!(
        "selection-sort",
        "Selection sort",
        SORTING,
        "SEL",
        sort::<1>
    ),
    tool!(
        "insertion-sort",
        "Insertion sort",
        SORTING,
        "INS",
        sort::<2>
    ),
    tool!("merge-sort", "Merge sort", SORTING, "MRG", sort::<3>),
    tool!("quick-sort", "Quick sort", SORTING, "QCK", sort::<4>),
    tool!(
        "quick-sort-3way",
        "Quick sort (3-way)",
        SORTING,
        "Q3W",
        sort::<5>
    ),
    tool!("heap-sort", "Heap sort", SORTING, "HEP", sort::<6>),
    tool!("counting-sort", "Counting sort", SORTING, "CNT", sort::<7>),
    tool!("sorting-race", "Sorting race", SORTING, "RCE", race),
    tool!("bfs", "Breadth-first search", GRAPHS, "BFS", path::<0>),
    tool!("dfs", "Depth-first search", GRAPHS, "DFS", path::<1>),
    tool!("dijkstra", "Dijkstra's algorithm", GRAPHS, "DIJ", path::<2>),
    tool!("a-star", "A* search", GRAPHS, "AST", path::<3>),
    tool!(
        "binary-search-tree",
        "Binary search tree",
        GRAPHS,
        "BST",
        new::<graphs::bst::SearchTree>
    ),
    tool!("kruskal", "Kruskal's algorithm", GRAPHS, "KRU", mst::<0>),
    tool!("prim", "Prim's algorithm", GRAPHS, "PRI", mst::<1>),
];

/// The tool with this id.
pub fn tool(id: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.id == id)
}

/// The tool whose id, alias or name matches what was typed (case-insensitive).
pub fn find_tool(query: &str) -> Option<&'static Tool> {
    let q = query.trim().to_lowercase();
    TOOLS
        .iter()
        .find(|t| t.alias.to_lowercase() == q || t.id == q || t.name.to_lowercase() == q)
}

/// Category names in catalogue order, each with its tools.
pub fn categories() -> Vec<(&'static str, Vec<&'static Tool>)> {
    let mut groups: Vec<(&'static str, Vec<&'static Tool>)> = Vec::new();
    for t in TOOLS {
        match groups.iter_mut().find(|(c, _)| *c == t.category) {
            Some((_, members)) => members.push(t),
            None => groups.push((t.category, vec![t])),
        }
    }
    groups
}

/// One instance of every tool, sharing one environment.
pub fn registry() -> Vec<Box<dyn Formula>> {
    let env = ToolEnv::new();
    TOOLS.iter().map(|t| (t.make)(&env)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_match_their_formulas() {
        let env = ToolEnv::new();
        for t in TOOLS {
            let f = (t.make)(&env);
            assert_eq!(f.id(), t.id);
            assert_eq!(f.name(), t.name, "{}", t.id);
            assert_eq!(f.category(), t.category, "{}", t.id);
            assert_eq!(tool(t.id).map(|x| x.id), Some(t.id));
            assert_eq!(find_tool(&t.alias.to_lowercase()).map(|x| x.id), Some(t.id));
        }
        let mut aliases: Vec<_> = TOOLS.iter().map(|t| t.alias).collect();
        aliases.sort();
        aliases.dedup();
        assert_eq!(aliases.len(), TOOLS.len());
    }

    #[test]
    fn ids_are_unique() {
        let all = registry();
        let mut ids: Vec<_> = all.iter().map(|f| f.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        assert!(ids.contains(&DEFAULT_ID));
    }

    #[test]
    fn categories_are_contiguous() {
        // The sidebar groups by first appearance; a category split across the
        // list would still group correctly, but keep the registry tidy anyway.
        let cats: Vec<_> = registry().iter().map(|f| f.category()).collect();
        let mut seen = Vec::new();
        for w in cats.windows(2) {
            if w[0] != w[1] {
                seen.push(w[0]);
                assert!(!seen.contains(&w[1]), "{} is split", w[1]);
            }
        }
    }

    /// Drawing a formula's controls must not change its parameters. egui's
    /// sliders round to `step_by` on every draw unless clamping is `Edits`.
    #[test]
    fn showing_controls_leaves_parameters_unchanged() {
        let ctx = egui::Context::default();
        let palette = crate::formula::Palette::dark();
        let env = ToolEnv::new();
        for tool in TOOLS {
            let mut f = (tool.make)(&env);
            let before = f.params();
            let mut out = ctx.run_ui(Default::default(), |ui| f.controls(ui, &palette));
            out.textures_delta.clear();
            assert_eq!(f.params(), before, "{}", tool.id);
        }
    }

    /// Every character a formula shows in its text must exist in egui's bundled
    /// fonts, or it renders as an empty box.
    #[test]
    fn all_text_has_glyphs() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        let font = egui::FontId::proportional(14.0);
        for f in registry() {
            let mut text = format!("{}{}{}{}", f.name(), f.category(), f.summary(), f.notes());
            for eq in f.equations() {
                text.push_str(&eq.text());
            }
            for d in f.derivations() {
                text.push_str(&d.title);
                for step in &d.steps {
                    text.push_str(&step.math.text());
                    text.push_str(&step.why);
                }
            }
            // The reference docs are drawn in the same font.
            text.push_str(crate::content::doc(f.id()).unwrap_or(""));
            let missing: String = ctx.fonts_mut(|fonts| {
                text.chars()
                    .filter(|&c| !c.is_whitespace() && !fonts.has_glyph(&font, c))
                    .collect()
            });
            assert!(missing.is_empty(), "{}: no glyph for {missing:?}", f.id());
        }
    }
}
