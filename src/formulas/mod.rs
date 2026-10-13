//! Every formula the app can visualize.
//!
//! Adding one: create a module here, implement [`Formula`] for its state,
//! and append it to [`registry`]. Nothing else in the app needs to change.

pub mod fourier;
pub mod lissajous;

use crate::formula::Formula;

/// The catalogue, in the order shown in the sidebar.
pub fn registry() -> Vec<Box<dyn Formula>> {
    vec![
        Box::new(fourier::Epicycles::default()),
        Box::new(lissajous::Lissajous::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let all = registry();
        let mut ids: Vec<_> = all.iter().map(|f| f.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
    }
}
