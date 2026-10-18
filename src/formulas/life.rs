//! Conway's Game of Life: a grid of cells, four rules, endless patterns.

use egui::{
    Align2, Color32, ColorImage, FontId, PointerButton, Pos2, Rect, Sense, Stroke, TextureHandle,
    TextureOptions, pos2, vec2,
};

use crate::draw::gradient;
use crate::formula::{Formula, MathText, Palette, Tone};
use crate::rng::Rng;

/// Which rule decides a cell's next state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Underpopulation,
    Survival,
    Overpopulation,
    Reproduction,
    /// A dead cell without exactly three neighbors stays dead.
    StaysDead,
}

impl Rule {
    pub fn of(alive: bool, neighbors: u8) -> Rule {
        match (alive, neighbors) {
            (true, 0..=1) => Rule::Underpopulation,
            (true, 2..=3) => Rule::Survival,
            (true, _) => Rule::Overpopulation,
            (false, 3) => Rule::Reproduction,
            (false, _) => Rule::StaysDead,
        }
    }

    pub fn next(self) -> bool {
        matches!(self, Rule::Survival | Rule::Reproduction)
    }

    fn describe(self) -> &'static str {
        match self {
            Rule::Underpopulation => "dies of underpopulation",
            Rule::Survival => "lives on",
            Rule::Overpopulation => "dies of overpopulation",
            Rule::Reproduction => "is born",
            Rule::StaysDead => "stays dead",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub alive: Vec<bool>,
    /// Edges wrap around (a torus) instead of being permanently dead.
    pub wrap: bool,
}

impl Grid {
    pub fn new(w: usize, h: usize, wrap: bool) -> Self {
        Self {
            w,
            h,
            alive: vec![false; w * h],
            wrap,
        }
    }

    pub fn get(&self, x: i64, y: i64) -> bool {
        let (w, h) = (self.w as i64, self.h as i64);
        let (x, y) = if self.wrap {
            (x.rem_euclid(w), y.rem_euclid(h))
        } else if (0..w).contains(&x) && (0..h).contains(&y) {
            (x, y)
        } else {
            return false;
        };
        self.alive[(y * w + x) as usize]
    }

    pub fn set(&mut self, x: usize, y: usize, on: bool) {
        if x < self.w && y < self.h {
            self.alive[y * self.w + x] = on;
        }
    }

    pub fn neighbors(&self, x: usize, y: usize) -> u8 {
        let (x, y) = (x as i64, y as i64);
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && self.get(x + dx, y + dy) {
                    n += 1;
                }
            }
        }
        n
    }

    /// The next generation: every cell updates at once.
    pub fn step(&self) -> Grid {
        let mut next = Grid::new(self.w, self.h, self.wrap);
        for y in 0..self.h {
            for x in 0..self.w {
                let alive = self.alive[y * self.w + x];
                next.alive[y * self.w + x] = Rule::of(alive, self.neighbors(x, y)).next();
            }
        }
        next
    }

    pub fn population(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    /// Stamp a pattern written as rows of `O` (alive) and `.` (dead), centered.
    pub fn stamp(&mut self, rows: &[&str]) {
        let ph = rows.len();
        let pw = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let (ox, oy) = (self.w.saturating_sub(pw) / 2, self.h.saturating_sub(ph) / 2);
        for (dy, row) in rows.iter().enumerate() {
            for (dx, c) in row.chars().enumerate() {
                if c == 'O' {
                    self.set(ox + dx, oy + dy, true);
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Pattern {
    Random,
    Glider,
    Gun,
    RPentomino,
    Acorn,
    Pulsar,
    Spaceship,
}

impl Pattern {
    const ALL: [Pattern; 7] = [
        Pattern::Random,
        Pattern::Glider,
        Pattern::Spaceship,
        Pattern::Pulsar,
        Pattern::Gun,
        Pattern::RPentomino,
        Pattern::Acorn,
    ];

    fn label(self) -> &'static str {
        match self {
            Pattern::Random => "Random soup",
            Pattern::Glider => "Glider",
            Pattern::Gun => "Glider gun",
            Pattern::RPentomino => "R-pentomino",
            Pattern::Acorn => "Acorn",
            Pattern::Pulsar => "Pulsar",
            Pattern::Spaceship => "Spaceship",
        }
    }

    fn rows(self) -> &'static [&'static str] {
        match self {
            Pattern::Random => &[],
            Pattern::Glider => &[".O.", "..O", "OOO"],
            Pattern::Spaceship => &[".O..O", "O....", "O...O", "OOOO."],
            Pattern::RPentomino => &[".OO", "OO.", ".O."],
            Pattern::Acorn => &[".O.....", "...O...", "OO..OOO"],
            Pattern::Pulsar => &[
                "..OOO...OOO..",
                ".............",
                "O....O.O....O",
                "O....O.O....O",
                "O....O.O....O",
                "..OOO...OOO..",
                ".............",
                "..OOO...OOO..",
                "O....O.O....O",
                "O....O.O....O",
                "O....O.O....O",
                ".............",
                "..OOO...OOO..",
            ],
            Pattern::Gun => &[
                "........................O...........",
                "......................O.O...........",
                "............OO......OO............OO",
                "...........O...O....OO............OO",
                "OO........O.....O...OO..............",
                "OO........O...O.OO....O.O...........",
                "..........O.....O.......O...........",
                "...........O...O....................",
                "............OO......................",
            ],
        }
    }
}

/// Generations a dead cell takes to fade from cyan back to the background.
const FADE: u16 = 28;
/// Grid widths for the size picker; height follows the stage's aspect.
const SIZES: [(usize, &str); 3] = [(48, "Small"), (96, "Medium"), (160, "Large")];

pub struct Life {
    grid: Grid,
    /// Generations alive (if alive) or since death (if dead); `u16::MAX` = never lived.
    age: Vec<u16>,
    cols: usize,
    pattern: Pattern,
    generation: u64,
    rate: f32,
    carry: f32,
    playing: bool,
    seed: u64,
    hover: Option<Pos2>,
    /// The cell last painted in the current stroke, to fill in fast drags.
    stroke_from: Option<(usize, usize)>,
    texture: Option<TextureHandle>,
}

impl Default for Life {
    fn default() -> Self {
        let mut l = Self {
            grid: Grid::new(1, 1, true),
            age: Vec::new(),
            cols: 96,
            pattern: Pattern::Random,
            generation: 0,
            rate: 12.0,
            carry: 0.0,
            playing: true,
            seed: 1,
            hover: None,
            stroke_from: None,
            texture: None,
        };
        l.load();
        l
    }
}

impl Life {
    fn rows(&self) -> usize {
        (self.cols as f32 * 0.6).round() as usize
    }

    fn load(&mut self) {
        let wrap = self.grid.wrap;
        let mut g = Grid::new(self.cols, self.rows(), wrap);
        if self.pattern == Pattern::Random {
            let mut rng = Rng::new(self.seed);
            for a in &mut g.alive {
                *a = rng.f32() < 0.28;
            }
        } else {
            g.stamp(self.pattern.rows());
        }
        self.age = g
            .alive
            .iter()
            .map(|&a| if a { 0 } else { u16::MAX })
            .collect();
        self.grid = g;
        self.generation = 0;
    }

    fn clear(&mut self) {
        self.grid = Grid::new(self.cols, self.rows(), self.grid.wrap);
        self.age = vec![u16::MAX; self.grid.alive.len()];
        self.generation = 0;
    }

    fn advance(&mut self) {
        let next = self.grid.step();
        for (k, age) in self.age.iter_mut().enumerate() {
            let (was, now) = (self.grid.alive[k], next.alive[k]);
            *age = match (was, now) {
                (true, true) => age.saturating_add(1),
                (_, true) => 0,
                (true, false) => 0,
                (false, false) if *age == u16::MAX => u16::MAX,
                (false, false) => age.saturating_add(1).min(FADE),
            };
        }
        self.grid = next;
        self.generation += 1;
    }

    /// The square-celled grid area, centered in the stage.
    fn grid_rect(&self, rect: Rect) -> Rect {
        let inner = rect.shrink(8.0);
        let cell = (inner.width() / self.grid.w as f32).min(inner.height() / self.grid.h as f32);
        Rect::from_center_size(
            rect.center(),
            vec2(cell * self.grid.w as f32, cell * self.grid.h as f32),
        )
    }

    fn cell_at(&self, rect: Rect, p: Pos2) -> Option<(usize, usize)> {
        let g = self.grid_rect(rect);
        if !g.contains(p) {
            return None;
        }
        let u = (p - g.min) / g.size();
        Some((
            ((u.x * self.grid.w as f32) as usize).min(self.grid.w - 1),
            ((u.y * self.grid.h as f32) as usize).min(self.grid.h - 1),
        ))
    }

    fn image(&self, palette: &Palette, dark: bool) -> ColorImage {
        let bg = if dark {
            Color32::from_rgb(8, 14, 20)
        } else {
            palette.card
        };
        let (newborn, adult) = if dark {
            (
                Color32::from_rgb(0xF0, 0xFF, 0xFF),
                Color32::from_rgb(0x4C, 0xD8, 0xE6),
            )
        } else {
            (
                Color32::from_rgb(0x06, 0x2A, 0x36),
                Color32::from_rgb(0x0E, 0x6E, 0x88),
            )
        };
        // Recently died: cyan, cooling to deep blue, then gone.
        let ghosts = if dark {
            [
                Color32::from_rgb(0x2C, 0x9C, 0xC8),
                Color32::from_rgb(0x1E, 0x3C, 0x8C),
                Color32::from_rgb(0x14, 0x1E, 0x46),
                bg,
            ]
        } else {
            [
                Color32::from_rgb(0x9C, 0xD8, 0xE4),
                Color32::from_rgb(0xB8, 0xCC, 0xEE),
                Color32::from_rgb(0xD8, 0xE2, 0xF2),
                bg,
            ]
        };
        let pixels = self
            .grid
            .alive
            .iter()
            .zip(&self.age)
            .map(|(&alive, &age)| {
                if alive {
                    gradient(&[newborn, adult], age as f32 / 6.0)
                } else if age == u16::MAX {
                    bg
                } else {
                    gradient(&ghosts, age as f32 / FADE as f32)
                }
            })
            .collect();
        ColorImage::new([self.grid.w, self.grid.h], pixels)
    }
}

/// What a sheet saves of a [`Life`].
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct State {
    /// Columns: one of the grid sizes (48, 96, 160).
    cols: usize,
    pattern: Pattern,
    /// The random pattern's seed.
    seed: u64,
    rate: f32,
    playing: bool,
    wrap: bool,
    /// The live cells, one string per row: `#` alive, `.` dead.
    cells: Vec<String>,
}

impl Life {
    fn state(&self) -> State {
        let g = &self.grid;
        State {
            cols: self.cols,
            pattern: self.pattern,
            seed: self.seed,
            rate: self.rate,
            playing: self.playing,
            wrap: g.wrap,
            cells: crate::state::grid_rows(
                g.w,
                g.h,
                |x, y| {
                    if g.alive[y * g.w + x] { '#' } else { '.' }
                },
            ),
        }
    }

    fn apply(&mut self, s: State) {
        if SIZES.iter().any(|&(c, _)| c == s.cols) {
            self.cols = s.cols;
        }
        self.pattern = s.pattern;
        self.seed = s.seed;
        self.rate = s.rate.clamp(1.0, 60.0);
        self.playing = s.playing;
        self.grid.wrap = s.wrap;
        self.load();
        if let Some(cells) = crate::state::parse_rows(&s.cells, self.grid.w, self.grid.h) {
            for (k, ch) in cells.iter().flatten().enumerate() {
                let alive = *ch == '#';
                self.grid.alive[k] = alive;
                self.age[k] = if alive { 0 } else { u16::MAX };
            }
        }
    }
}

impl Formula for Life {
    crate::state_hooks!();

    fn id(&self) -> &'static str {
        "game-of-life"
    }

    fn name(&self) -> &'static str {
        "Conway's Game of Life"
    }

    fn category(&self) -> &'static str {
        super::FRACTALS
    }

    fn summary(&self) -> &'static str {
        "A grid of cells, each alive or dead. Every generation each cell counts its eight \
         neighbors and four rules decide its fate. Gliders, oscillators and guns emerge on \
         their own. Click or drag to draw cells, right-drag to erase, hover to see the rule."
    }

    fn notes(&self) -> &'static str {
        "Cellular automata model forest fires, epidemics, crystal growth and traffic; they \
         generate cave maps and textures in games, and the Game of Life itself can compute \
         anything a computer can. Fading recently dead cells, as here, is a common trick in \
         generative art."
    }

    fn equations(&self) -> Vec<MathText> {
        vec![
            MathText::new().t("n = live cells among the 8 neighbors"),
            MathText::new()
                .t("alive, n < 2: dies")
                .t("  (underpopulation)"),
            MathText::new()
                .t("alive, n = 2 or 3: lives on")
                .tone(Tone::X)
                .t("  (survival)"),
            MathText::new()
                .t("alive, n > 3: dies")
                .t("  (overpopulation)"),
            MathText::new()
                .t("dead, n = 3: born")
                .tone(Tone::Y)
                .t("  (reproduction)"),
        ]
    }

    fn stage_sense(&self) -> Sense {
        Sense::click_and_drag()
    }

    fn stage_height(&self, width: f32) -> f32 {
        width * 0.62
    }

    fn interact(&mut self, response: &egui::Response, rect: Rect) {
        self.hover = response.hover_pos();
        let Some(at) = response.interact_pointer_pos() else {
            return;
        };
        let down = response.is_pointer_button_down_on() || response.clicked();
        if !down {
            self.stroke_from = None;
            return;
        }
        let erase = response
            .ctx
            .input(|i| i.pointer.button_down(PointerButton::Secondary))
            || response.secondary_clicked();
        let Some(to) = self.cell_at(rect, at) else {
            return;
        };
        // Paint every cell on the way from the last one, so fast drags leave
        // no gaps.
        let from = self.stroke_from.unwrap_or(to);
        let steps = from.0.abs_diff(to.0).max(from.1.abs_diff(to.1)).max(1);
        for k in 0..=steps {
            let t = k as f32 / steps as f32;
            let x = (from.0 as f32 + (to.0 as f32 - from.0 as f32) * t).round() as usize;
            let y = (from.1 as f32 + (to.1 as f32 - from.1 as f32) * t).round() as usize;
            self.grid.set(x, y, !erase);
            self.age[y * self.grid.w + x] = if erase { u16::MAX } else { 0 };
        }
        self.stroke_from = Some(to);
    }

    fn paint(&mut self, painter: &egui::Painter, rect: Rect, palette: &Palette, dt: f32) {
        if self.playing {
            self.carry += self.rate * dt.min(0.1);
            while self.carry >= 1.0 {
                self.carry -= 1.0;
                self.advance();
            }
        }
        let dark = palette.paper.r() < 128;
        let image = self.image(palette, dark);
        match &mut self.texture {
            Some(t) => t.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(painter.ctx().load_texture(
                    "life",
                    image,
                    TextureOptions::NEAREST,
                ))
            }
        }
        let g = self.grid_rect(rect);
        if let Some(t) = &self.texture {
            painter.image(
                t.id(),
                g,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let cell = g.width() / self.grid.w as f32;
        if cell >= 7.0 {
            let line = Stroke::new(
                1.0,
                palette.rule.gamma_multiply(if dark { 0.35 } else { 0.6 }),
            );
            for x in 0..=self.grid.w {
                let px = g.left() + x as f32 * cell;
                painter.line_segment([pos2(px, g.top()), pos2(px, g.bottom())], line);
            }
            for y in 0..=self.grid.h {
                let py = g.top() + y as f32 * cell;
                painter.line_segment([pos2(g.left(), py), pos2(g.right(), py)], line);
            }
        }
        painter.rect_stroke(
            g,
            0.0,
            Stroke::new(1.0, palette.rule),
            egui::StrokeKind::Outside,
        );

        // Hover: outline the neighborhood and say which rule applies.
        if let Some((x, y)) = self.hover.and_then(|p| self.cell_at(rect, p)) {
            let cell_rect = |cx: i64, cy: i64| {
                Rect::from_min_size(
                    g.min + vec2(cx as f32 * cell, cy as f32 * cell),
                    vec2(cell, cell),
                )
            };
            let (xi, yi) = (x as i64, y as i64);
            let hood = cell_rect(xi - 1, yi - 1).union(cell_rect(xi + 1, yi + 1));
            painter.rect_stroke(
                hood,
                1.0,
                Stroke::new(1.5, palette.y),
                egui::StrokeKind::Outside,
            );
            painter.rect_stroke(
                cell_rect(xi, yi),
                0.0,
                Stroke::new(1.5, palette.ink),
                egui::StrokeKind::Outside,
            );
            let alive = self.grid.get(xi, yi);
            let n = self.grid.neighbors(x, y);
            let rule = Rule::of(alive, n);
            let text = format!(
                "{} cell, {n} live neighbor{}: {}",
                if alive { "Live" } else { "Dead" },
                if n == 1 { "" } else { "s" },
                rule.describe()
            );
            let color = match rule {
                Rule::Survival => palette.x,
                Rule::Reproduction => palette.y,
                _ => palette.ink,
            };
            let galley = painter.layout_no_wrap(text, FontId::proportional(13.0), color);
            let above = hood.top() - 8.0 - galley.size().y > rect.top();
            let anchor = if above {
                pos2(hood.center().x, hood.top() - 6.0)
            } else {
                pos2(hood.center().x, hood.bottom() + 6.0 + galley.size().y)
            };
            let mut r = Align2::CENTER_BOTTOM.anchor_size(anchor, galley.size());
            let shift =
                (rect.left() + 8.0 - r.left()).max(0.0) - (r.right() - rect.right() + 8.0).max(0.0);
            r = r.translate(vec2(shift, 0.0));
            painter.rect_filled(r.expand(5.0), 5.0, palette.card);
            painter.rect_stroke(
                r.expand(5.0),
                5.0,
                Stroke::new(1.0, palette.rule),
                egui::StrokeKind::Inside,
            );
            painter.galley(r.min, galley, color);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui, _palette: &Palette) {
        egui::Grid::new("life-controls")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Start");
                ui.horizontal_wrapped(|ui| {
                    for p in Pattern::ALL {
                        if ui.selectable_label(self.pattern == p, p.label()).clicked() {
                            self.pattern = p;
                            self.seed += 1;
                            self.load();
                        }
                    }
                });
                ui.end_row();
                ui.label("Grid");
                ui.horizontal(|ui| {
                    for (cols, label) in SIZES {
                        if ui.selectable_label(self.cols == cols, label).clicked()
                            && self.cols != cols
                        {
                            self.cols = cols;
                            self.load();
                        }
                    }
                    let mut wrap = self.grid.wrap;
                    if ui
                        .checkbox(&mut wrap, "Wrap edges")
                        .on_hover_text("Cells on one edge count the opposite edge as neighbors.")
                        .changed()
                    {
                        self.grid.wrap = wrap;
                    }
                });
                ui.end_row();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.rate, 1.0..=60.0).suffix(" gen/s"));
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("Step").clicked() {
                self.playing = false;
                self.advance();
            }
            if ui.button("Restart").clicked() {
                self.load();
            }
            if ui.button("Clear").clicked() {
                self.clear();
                self.playing = false;
            }
            ui.label(format!(
                "Generation {}  ·  {} alive",
                self.generation,
                self.grid.population()
            ));
        });
    }

    fn details(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.label(
            egui::RichText::new(
                "Live cells start bright and settle to cyan; cells that just died glow and \
                 cool to deep blue over a few dozen generations, so the trails show where \
                 life has been. The R-pentomino and the acorn are tiny seeds that take \
                 over a thousand generations to settle.",
            )
            .color(palette.muted),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn painted_cells_survive_a_save() {
        let mut life = Life::default();
        life.clear();
        life.grid.set(3, 4, true);
        let saved = life.save_state().unwrap();
        let mut back = Life::default();
        back.load_state(&saved);
        assert!(back.grid.get(3, 4));
        assert_eq!(back.grid.alive.iter().filter(|&&a| a).count(), 1);
        // Rows of the wrong width are ignored: the pattern is drawn instead.
        let mut bad = saved.clone();
        bad["cells"] = serde_json::json!(["#.#"]);
        back.load_state(&bad);
        assert!(back.grid.alive.iter().filter(|&&a| a).count() > 1);
    }

    fn grid(w: usize, h: usize, rows: &[&str]) -> Grid {
        let mut g = Grid::new(w, h, true);
        g.stamp(rows);
        g
    }

    #[test]
    fn rules() {
        assert_eq!(Rule::of(true, 1), Rule::Underpopulation);
        assert_eq!(Rule::of(true, 2), Rule::Survival);
        assert_eq!(Rule::of(true, 3), Rule::Survival);
        assert_eq!(Rule::of(true, 4), Rule::Overpopulation);
        assert_eq!(Rule::of(false, 3), Rule::Reproduction);
        assert_eq!(Rule::of(false, 2), Rule::StaysDead);
        assert!(Rule::Reproduction.next() && !Rule::Overpopulation.next());
    }

    #[test]
    fn block_is_still() {
        let g = grid(6, 6, &["OO", "OO"]);
        assert_eq!(g.step(), g);
    }

    #[test]
    fn blinker_has_period_two() {
        let g = grid(5, 5, &["OOO"]);
        let once = g.step();
        assert_ne!(once, g);
        assert_eq!(once.population(), 3);
        assert_eq!(once.step(), g);
    }

    #[test]
    fn glider_moves_diagonally() {
        let g = grid(10, 10, Pattern::Glider.rows());
        let mut later = g.clone();
        for _ in 0..4 {
            later = later.step();
        }
        // Same shape, one cell down and to the right.
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(later.get(x + 1, y + 1), g.get(x, y), "at ({x}, {y})");
            }
        }
    }

    #[test]
    fn edges_wrap_or_not() {
        let mut g = Grid::new(4, 4, true);
        g.set(3, 3, true);
        assert_eq!(g.neighbors(0, 0), 1);
        g.wrap = false;
        assert_eq!(g.neighbors(0, 0), 0);
    }

    #[test]
    fn gun_fires_gliders() {
        let mut g = Grid::new(120, 80, false);
        g.stamp(Pattern::Gun.rows());
        let start = g.population();
        for _ in 0..90 {
            g = g.step();
        }
        // The gun repeats every 30 generations and adds one 5-cell glider each time.
        assert_eq!(g.population(), start + 15);
    }

    #[test]
    fn ages_track_birth_and_death() {
        let mut l = Life {
            cols: 10,
            pattern: Pattern::Glider,
            ..Life::default()
        };
        l.load();
        l.advance();
        let cells = || l.grid.alive.iter().zip(&l.age);
        // A glider loses two cells and gains two each generation.
        assert_eq!(
            cells().filter(|&(&alive, &age)| !alive && age == 0).count(),
            2
        );
        assert_eq!(
            cells().filter(|&(&alive, &age)| alive && age == 0).count(),
            2
        );
        assert_eq!(
            cells().filter(|&(&alive, &age)| alive && age == 1).count(),
            3
        );
    }
}
