//! The list being sorted, shared by every sorting page so algorithms can be
//! compared on exactly the same input.

use std::cell::RefCell;
use std::rc::Rc;

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distribution {
    Random,
    NearlySorted,
    Reversed,
    FewUnique,
}

impl Distribution {
    pub const ALL: [Distribution; 4] = [
        Distribution::Random,
        Distribution::NearlySorted,
        Distribution::Reversed,
        Distribution::FewUnique,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Distribution::Random => "Random",
            Distribution::NearlySorted => "Nearly sorted",
            Distribution::Reversed => "Reversed",
            Distribution::FewUnique => "Few unique",
        }
    }
}

/// Values `1..=n` arranged according to `dist`.
pub fn generate(n: usize, dist: Distribution, seed: u64) -> Vec<u32> {
    let mut rng = Rng::new(seed);
    let mut v: Vec<u32> = (1..=n as u32).collect();
    let shuffle = |v: &mut Vec<u32>, rng: &mut Rng| {
        // Fisher–Yates.
        for i in (1..v.len()).rev() {
            v.swap(i, rng.index(i + 1));
        }
    };
    match dist {
        Distribution::Random => shuffle(&mut v, &mut rng),
        Distribution::NearlySorted => {
            for _ in 0..(n / 10).max(1) {
                if n > 1 {
                    let i = rng.index(n - 1);
                    v.swap(i, i + 1);
                }
            }
        }
        Distribution::Reversed => v.reverse(),
        Distribution::FewUnique => {
            let levels = [n / 4, n / 2, 3 * n / 4, n].map(|x| x.max(1) as u32);
            for x in &mut v {
                *x = levels[rng.index(levels.len())];
            }
        }
    }
    v
}

pub struct Dataset {
    pub n: usize,
    pub dist: Distribution,
    seed: u64,
    values: Vec<u32>,
    /// Bumped on every change so each page knows to re-record its steps.
    generation: u64,
}

impl Default for Dataset {
    fn default() -> Self {
        Self::new()
    }
}

impl Dataset {
    pub fn new() -> Self {
        let mut d = Self {
            n: 40,
            dist: Distribution::Random,
            seed: 1,
            values: Vec::new(),
            generation: 0,
        };
        d.regenerate();
        d
    }

    pub fn values(&self) -> &[u32] {
        &self.values
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Rebuild the list from the current size and distribution.
    pub fn regenerate(&mut self) {
        self.values = generate(self.n, self.dist, self.seed);
        self.generation += 1;
    }

    /// A fresh random arrangement.
    pub fn shuffle(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.regenerate();
    }
}

pub type Shared = Rc<RefCell<Dataset>>;

pub fn shared() -> Shared {
    Rc::new(RefCell::new(Dataset::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_is_a_permutation() {
        let mut v = generate(50, Distribution::Random, 3);
        assert_ne!(v, (1..=50).collect::<Vec<_>>());
        v.sort();
        assert_eq!(v, (1..=50).collect::<Vec<_>>());
    }

    #[test]
    fn shapes() {
        assert_eq!(generate(5, Distribution::Reversed, 0), vec![5, 4, 3, 2, 1]);
        let nearly = generate(100, Distribution::NearlySorted, 1);
        let out_of_place = nearly
            .iter()
            .enumerate()
            .filter(|&(i, &x)| x != i as u32 + 1)
            .count();
        assert!(out_of_place > 0 && out_of_place <= 20);
        let few = generate(40, Distribution::FewUnique, 2);
        let mut distinct = few.clone();
        distinct.sort();
        distinct.dedup();
        assert!(distinct.len() <= 4);
    }

    #[test]
    fn tiny_sizes() {
        for dist in Distribution::ALL {
            assert_eq!(generate(0, dist, 1).len(), 0);
            assert_eq!(generate(1, dist, 1), vec![1]);
        }
    }

    #[test]
    fn changes_bump_generation() {
        let mut d = Dataset::new();
        let g = d.generation();
        d.shuffle();
        assert!(d.generation() > g);
    }
}
