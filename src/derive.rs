//! Derivations: a formula explained as a sequence of steps, each an equation
//! and the reason it follows from the one before. The Inspector's Derive tab
//! reveals them one step at a time.

use crate::formula::MathText;

/// One line of a derivation.
#[derive(Clone, Debug)]
pub struct Step {
    pub math: MathText,
    /// Why this line follows from the previous one, in a short sentence.
    pub why: String,
}

impl Step {
    pub fn new(math: MathText, why: impl Into<String>) -> Self {
        Self {
            math,
            why: why.into(),
        }
    }
}

/// A titled sequence of steps, e.g. "From Pythagoras".
#[derive(Clone, Debug)]
pub struct Derivation {
    pub title: String,
    pub steps: Vec<Step>,
}

impl Derivation {
    pub fn new(title: impl Into<String>, steps: Vec<Step>) -> Self {
        Self {
            title: title.into(),
            steps,
        }
    }
}

/// Keeps a derivation pick and a revealed-step index inside the derivations
/// whose step counts are `lens`; `(0, 0)` when there are none.
pub fn clamp_steps(pick: usize, step: usize, lens: &[usize]) -> (usize, usize) {
    let Some(last) = lens.len().checked_sub(1) else {
        return (0, 0);
    };
    let pick = pick.min(last);
    (pick, step.min(lens[pick].saturating_sub(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_and_steps_stay_in_range() {
        assert_eq!(clamp_steps(3, 9, &[]), (0, 0));
        assert_eq!(clamp_steps(0, 9, &[4]), (0, 3));
        assert_eq!(clamp_steps(5, 1, &[4, 2]), (1, 1));
        assert_eq!(clamp_steps(1, 0, &[4, 0]), (1, 0));
        assert_eq!(clamp_steps(0, 2, &[7, 2]), (0, 2));
    }
}
