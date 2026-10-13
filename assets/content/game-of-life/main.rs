//! Conway's Game of Life (B3/S23): a cell is born with exactly 3 live neighbors and
//! survives with 2 or 3; every other cell dies or stays dead.
//!
//! A random soup on a wrap-around grid. Live cells start bright and settle to cyan; cells
//! that just died glow and cool to deep blue, leaving fading trails. Click or drag to draw
//! cells, right-drag to erase, hover to see which rule applies.
//! Keys: space pauses, R makes a new soup, C clears.
//!
//! Cargo.toml:
//!     [dependencies]
//!     macroquad = "0.4"
//! Run: cargo run --release

use macroquad::prelude::*;

const COLS: usize = 100;
const ROWS: usize = 62;
const CELL: f32 = 8.0;
const FADE: u16 = 28; // generations a dead cell takes to fade back to the background
const NEVER: u16 = u16::MAX; // age of a cell that has never been alive
const GENS_PER_SECOND: f32 = 12.0;

struct Life {
    alive: Vec<bool>,
    /// Generations alive (if alive) or since death (if dead).
    age: Vec<u16>,
}

impl Life {
    fn random_soup() -> Self {
        let alive: Vec<bool> = (0..COLS * ROWS)
            .map(|_| macroquad::rand::gen_range(0.0, 1.0) < 0.28)
            .collect();
        let age = alive.iter().map(|&a| if a { 0 } else { NEVER }).collect();
        Self { alive, age }
    }

    /// Live cells among the 8 neighbors; coordinates wrap around (a torus).
    fn neighbors(&self, x: usize, y: usize) -> u8 {
        let mut n = 0;
        for dy in [ROWS - 1, 0, 1] {
            for dx in [COLS - 1, 0, 1] {
                if (dx, dy) != (0, 0) && self.alive[(y + dy) % ROWS * COLS + (x + dx) % COLS] {
                    n += 1;
                }
            }
        }
        n
    }

    /// Every cell updates at once, from the old grid into a new one.
    fn advance(&mut self) {
        let mut next = vec![false; COLS * ROWS];
        for y in 0..ROWS {
            for x in 0..COLS {
                let k = y * COLS + x;
                next[k] = next_state(self.alive[k], self.neighbors(x, y));
                let age = &mut self.age[k];
                *age = match (self.alive[k], next[k]) {
                    (true, true) => age.saturating_add(1).min(NEVER - 1),
                    (was, now) if was != now => 0,
                    _ if *age == NEVER => NEVER,
                    _ => (*age + 1).min(FADE),
                };
            }
        }
        self.alive = next;
    }

    fn set(&mut self, x: usize, y: usize, on: bool) {
        self.alive[y * COLS + x] = on;
        self.age[y * COLS + x] = if on { 0 } else { NEVER };
    }
}

/// The four rules in one line: birth on 3, survival on 2 or 3.
fn next_state(alive: bool, n: u8) -> bool {
    n == 3 || (alive && n == 2)
}

fn describe(alive: bool, n: u8) -> &'static str {
    match (alive, n) {
        (true, 0..=1) => "dies of underpopulation",
        (true, 2..=3) => "lives on",
        (true, _) => "dies of overpopulation",
        (false, 3) => "is born",
        (false, _) => "stays dead",
    }
}

/// Piecewise-linear color ramp through `stops`, t in 0..1.
fn gradient(stops: &[Color], t: f32) -> Color {
    let s = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let k = (s as usize).min(stops.len() - 2);
    let (a, b, f) = (stops[k], stops[k + 1], s - k as f32);
    Color::new(
        a.r + (b.r - a.r) * f,
        a.g + (b.g - a.g) * f,
        a.b + (b.b - a.b) * f,
        1.0,
    )
}

#[macroquad::main("Conway's Game of Life")]
async fn main() {
    let bg = Color::from_rgba(8, 14, 20, 255);
    let live = [
        Color::from_rgba(240, 255, 255, 255),
        Color::from_rgba(76, 216, 230, 255),
    ];
    let ghosts = [
        Color::from_rgba(44, 156, 200, 255),
        Color::from_rgba(30, 60, 140, 255),
        Color::from_rgba(20, 30, 70, 255),
        bg,
    ];
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    let mut life = Life::random_soup();
    let (mut playing, mut generation, mut carry) = (true, 0u64, 0.0f32);
    let mut stroke_from: Option<(usize, usize)> = None;

    loop {
        let (mx, my) = mouse_position();
        let hover = (mx >= 0.0 && my >= 0.0 && mx < COLS as f32 * CELL && my < ROWS as f32 * CELL)
            .then(|| ((mx / CELL) as usize, (my / CELL) as usize));
        let (left, right) = (
            is_mouse_button_down(MouseButton::Left),
            is_mouse_button_down(MouseButton::Right),
        );
        match hover {
            // Paint every cell on the line from the last one, so fast drags leave no gaps.
            Some(to) if left || right => {
                let from = stroke_from.unwrap_or(to);
                let steps = from.0.abs_diff(to.0).max(from.1.abs_diff(to.1)).max(1);
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let x = (from.0 as f32 + (to.0 as f32 - from.0 as f32) * t).round() as usize;
                    let y = (from.1 as f32 + (to.1 as f32 - from.1 as f32) * t).round() as usize;
                    life.set(x, y, left);
                }
                stroke_from = Some(to);
            }
            _ => stroke_from = None,
        }
        if is_key_pressed(KeyCode::Space) {
            playing = !playing;
        }
        if is_key_pressed(KeyCode::R) {
            life = Life::random_soup();
            generation = 0;
        }
        if is_key_pressed(KeyCode::C) {
            life.alive.fill(false);
            life.age.fill(NEVER);
            generation = 0;
        }
        if playing {
            carry += GENS_PER_SECOND * get_frame_time().min(0.1);
        }
        while carry >= 1.0 {
            carry -= 1.0;
            life.advance();
            generation += 1;
        }

        clear_background(bg);
        for (k, (&alive, &age)) in life.alive.iter().zip(&life.age).enumerate() {
            if !alive && age == NEVER {
                continue;
            }
            let color = if alive {
                gradient(&live, age as f32 / 6.0)
            } else {
                gradient(&ghosts, age as f32 / FADE as f32)
            };
            draw_rectangle(
                (k % COLS) as f32 * CELL,
                (k / COLS) as f32 * CELL,
                CELL,
                CELL,
                color,
            );
        }

        // Hover: outline the 3 x 3 neighborhood and name the rule that decides the cell.
        let mut note =
            "click/drag: draw  right-drag: erase  space: pause  R: soup  C: clear".to_string();
        if let Some((x, y)) = hover {
            let (alive, n) = (life.alive[y * COLS + x], life.neighbors(x, y));
            let (px, py) = ((x as f32 - 1.0) * CELL, (y as f32 - 1.0) * CELL);
            draw_rectangle_lines(px, py, 3.0 * CELL, 3.0 * CELL, 2.0, GOLD);
            let state = if alive { "live" } else { "dead" };
            note = format!("{state} cell, {n} live neighbors: {}", describe(alive, n));
        }
        let population = life.alive.iter().filter(|&&a| a).count();
        let paused = if playing { "" } else { "  (paused)" };
        let text = format!("gen {generation}  alive {population}{paused}   {note}");
        draw_text(&text, 8.0, ROWS as f32 * CELL + 22.0, 18.0, LIGHTGRAY);
        next_frame().await;
    }
}
