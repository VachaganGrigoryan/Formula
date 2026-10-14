//! Graph and tree algorithms recorded once and replayed step by step.

pub mod grid;

use std::cell::RefCell;
use std::rc::Rc;

/// What viewports of the graph tools on one board share, so algorithms can
/// be compared on exactly the same input.
#[derive(Clone)]
pub struct Session {
    pub grid: Rc<RefCell<grid::Grid>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            grid: Rc::new(RefCell::new(grid::Grid::new(grid::GRID_W, grid::GRID_H))),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
