//! Graph and tree algorithms recorded once and replayed step by step.

pub mod bst;
pub mod grid;
pub mod mst;
pub mod path;
pub mod pathfinding;
pub mod tree;

use std::cell::RefCell;
use std::rc::Rc;

/// What viewports of the graph tools on one board share, so algorithms can
/// be compared on exactly the same input.
#[derive(Clone)]
pub struct Session {
    pub grid: Rc<RefCell<grid::Grid>>,
    pub graph: Rc<RefCell<mst::Graph>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            grid: Rc::new(RefCell::new(grid::Grid::new(grid::GRID_W, grid::GRID_H))),
            graph: Rc::new(RefCell::new(mst::Graph::random(14, 7))),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
