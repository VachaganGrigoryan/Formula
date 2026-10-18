//! Derivations: a formula explained as a sequence of steps, each an equation
//! and the reason it follows from the one before. The Inspector's Derive tab
//! reveals them one step at a time.

use crate::expr::{self, BinOp, Expr, Func};
use crate::formula::{Formula, MathText};

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

/// One equation to work through with the current values, e.g. `d` with
/// `sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)` and the two points' coordinates.
pub struct Substitution {
    /// The left-hand side, as the equations show it.
    pub lhs: MathText,
    pub expr: Expr,
    /// Each variable's current value.
    pub values: Vec<(String, f64)>,
}

impl Substitution {
    /// `src` must parse; the sources are written in the tools, and a test
    /// checks every one.
    pub fn new(lhs: MathText, src: &str, values: &[(&str, f64)]) -> Self {
        Self {
            lhs,
            expr: expr::parse_expr(src).unwrap_or_else(|e| panic!("{src}: {}", e.message)),
            values: values.iter().map(|&(n, v)| (n.to_string(), v)).collect(),
        }
    }
}

/// One step of working: the expression so far, the part just computed, and
/// what was done.
#[derive(Clone, Debug, PartialEq)]
pub struct Reduction {
    pub expr: Expr,
    /// Child indices from the root to the value just computed.
    pub focus: Option<Vec<usize>>,
    pub why: String,
}

/// Works `e` out with the values from `value` (constants `π` and `e` are
/// known): first every variable becomes its value, then each step computes
/// the innermost-leftmost operation whose operands are all numbers, until
/// one number is left. A negated number is folded into the step that made it.
pub fn substitution_steps(e: &Expr, value: &dyn Fn(&str) -> Option<f64>) -> Vec<Reduction> {
    let mut cur = substitute(e, value);
    let mut steps = vec![Reduction {
        expr: cur.clone(),
        focus: None,
        why: "substitute the current values".into(),
    }];
    let mut path = Vec::new();
    while let Some(why) = reduce(&mut cur, &mut path) {
        steps.push(Reduction {
            expr: cur.clone(),
            focus: Some(path.clone()),
            why,
        });
        path.clear();
    }
    steps
}

fn substitute(e: &Expr, value: &dyn Fn(&str) -> Option<f64>) -> Expr {
    match e {
        Expr::Num(_) => e.clone(),
        Expr::Var(n) => match value(n).or_else(|| expr::constant(n)) {
            Some(v) => Expr::Num(v),
            None => e.clone(),
        },
        Expr::Neg(a) => fold(Expr::Neg(Box::new(substitute(a, value)))),
        Expr::Bin(op, a, b) => Expr::Bin(
            *op,
            Box::new(substitute(a, value)),
            Box::new(substitute(b, value)),
        ),
        Expr::Call(f, args) => Expr::Call(*f, args.iter().map(|a| substitute(a, value)).collect()),
    }
}

/// `-(number)` is just a negative number.
fn fold(e: Expr) -> Expr {
    match e {
        Expr::Neg(a) => match *a {
            Expr::Num(v) => Expr::Num(-v),
            a => Expr::Neg(Box::new(a)),
        },
        e => e,
    }
}

/// Computes the innermost-leftmost operation on numbers in place, leaving
/// its path in `path`. Returns what was done, or `None` when nothing is left.
fn reduce(e: &mut Expr, path: &mut Vec<usize>) -> Option<String> {
    let children: Vec<&mut Expr> = match e {
        Expr::Num(_) | Expr::Var(_) => return None,
        Expr::Neg(a) => vec![&mut **a],
        Expr::Bin(_, a, b) => vec![&mut **a, &mut **b],
        Expr::Call(_, args) => args.iter_mut().collect(),
    };
    for (i, child) in children.into_iter().enumerate() {
        path.push(i);
        if let Some(why) = reduce(child, path) {
            // A negation whose operand just became a number folds into this step.
            if matches!(e, Expr::Neg(_)) {
                *e = fold(std::mem::replace(e, Expr::Num(0.0)));
                if matches!(e, Expr::Num(_)) {
                    path.pop();
                }
            }
            return Some(why);
        }
        path.pop();
    }
    let all_numbers = match &*e {
        Expr::Bin(_, a, b) => matches!((&**a, &**b), (Expr::Num(_), Expr::Num(_))),
        Expr::Call(_, args) => args.iter().all(|a| matches!(a, Expr::Num(_))),
        _ => false,
    };
    if !all_numbers {
        return None;
    }
    let why = describe(e);
    *e = Expr::Num(expr::eval(e, &|_| None));
    Some(why)
}

/// What computing `e` (an operation on numbers) is called.
fn describe(e: &Expr) -> String {
    let word = match e {
        Expr::Bin(BinOp::Add, ..) => "add",
        Expr::Bin(BinOp::Sub, ..) => "subtract",
        Expr::Bin(BinOp::Mul, ..) => "multiply",
        Expr::Bin(BinOp::Div, ..) => "divide",
        Expr::Bin(BinOp::Pow, _, b) if **b == Expr::Num(2.0) => "square",
        Expr::Bin(BinOp::Pow, ..) => "power",
        Expr::Call(f, _) => match f {
            Func::Sin => "sine",
            Func::Cos => "cosine",
            Func::Tan => "tangent",
            Func::Asin => "arcsine",
            Func::Acos => "arccosine",
            Func::Atan => "arctangent",
            Func::Sqrt => "square root",
            Func::Abs => "absolute value",
            Func::Exp => "exponential",
            Func::Ln => "natural logarithm",
            Func::Log => "logarithm base 10",
            Func::Floor => "round down",
            Func::Ceil => "round up",
            Func::Min => "minimum",
            Func::Max => "maximum",
        },
        _ => "compute",
    };
    word.into()
}

/// "Plug in current values": each substitution as its formula, then worked
/// out one operation per step. `None` when there is nothing to plug in.
pub fn plug_in(subs: &[Substitution]) -> Option<Derivation> {
    if subs.is_empty() {
        return None;
    }
    let mut steps = Vec::new();
    for s in subs {
        let eq = |m: MathText| s.lhs.clone().append(MathText::new().t(" = ")).append(m);
        steps.push(Step::new(eq(expr::expr_math(&s.expr, None)), "the formula"));
        let value = |n: &str| s.values.iter().find(|(name, _)| name == n).map(|p| p.1);
        for r in substitution_steps(&s.expr, &value) {
            steps.push(Step::new(
                eq(expr::expr_math(&r.expr, r.focus.as_deref())),
                r.why,
            ));
        }
    }
    Some(Derivation::new("Plug in current values", steps))
}

/// What the Derive tab shows for `f`: the plug-in working first, then the
/// authored derivations.
pub fn all_derivations(f: &dyn Formula) -> Vec<Derivation> {
    plug_in(&f.substitutions())
        .into_iter()
        .chain(f.derivations())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::{Piece, Tone};
    use crate::rng::Rng;

    fn ops(e: &Expr) -> usize {
        match e {
            Expr::Num(_) | Expr::Var(_) => 0,
            Expr::Neg(a) => ops(a),
            Expr::Bin(_, a, b) => 1 + ops(a) + ops(b),
            Expr::Call(_, args) => 1 + args.iter().map(ops).sum::<usize>(),
        }
    }

    fn same(a: f64, b: f64) -> bool {
        (a.is_nan() && b.is_nan()) || a == b || (a - b).abs() <= 1e-9 * a.abs().max(b.abs())
    }

    #[test]
    fn every_step_has_the_same_value_and_ends_in_a_number() {
        let mut rng = Rng::new(21);
        for _ in 0..50 {
            let e = expr::random_expr(&mut rng, 4);
            let v = 0.5 + rng.f32() as f64 * 3.0;
            let value = |_: &str| Some(v);
            let steps = substitution_steps(&e, &value);
            let want = expr::eval(&e, &value);
            for s in &steps {
                assert!(
                    same(expr::eval(&s.expr, &|_| None), want),
                    "{}",
                    expr::print(&e)
                );
            }
            assert!(matches!(steps.last().unwrap().expr, Expr::Num(_)));
            assert_eq!(steps.len(), 1 + ops(&e), "{}", expr::print(&e));
        }
    }

    #[test]
    fn distance_is_worked_out_in_order() {
        let e = expr::parse_expr("sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)").unwrap();
        let vals = [("x_1", 0.0), ("y_1", 0.0), ("x_2", 3.0), ("y_2", 4.0)];
        let value = |n: &str| vals.iter().find(|p| p.0 == n).map(|p| p.1);
        let steps = substitution_steps(&e, &value);
        let whys: Vec<&str> = steps.iter().map(|s| s.why.as_str()).collect();
        assert_eq!(
            whys,
            [
                "substitute the current values",
                "subtract",
                "square",
                "subtract",
                "square",
                "add",
                "square root"
            ]
        );
        assert_eq!(steps.last().unwrap().expr, Expr::Num(5.0));
    }

    #[test]
    fn negations_fold_into_the_step_that_made_them() {
        let e = expr::parse_expr("-(a + b) * 2").unwrap();
        let steps = substitution_steps(&e, &|_| Some(1.0));
        let texts: Vec<String> = steps
            .iter()
            .map(|s| expr::expr_math(&s.expr, None).text())
            .collect();
        assert_eq!(texts, ["−(1 + 1) · 2", "−2 · 2", "−4"]);
    }

    #[test]
    fn degenerate_values_still_finish() {
        let e = expr::parse_expr("(x - a) / (b - a)").unwrap();
        let steps = substitution_steps(&e, &|n| Some(if n == "x" { 2.0 } else { 1.0 }));
        let last = expr::expr_math(&steps.last().unwrap().expr, None).text();
        assert_eq!(last, "∞");
        let steps = substitution_steps(&e, &|_| Some(1.0));
        assert_eq!(
            expr::expr_math(&steps.last().unwrap().expr, None).text(),
            "undefined"
        );
    }

    #[test]
    fn the_computed_part_is_highlighted() {
        let e = expr::parse_expr("a * b + c").unwrap();
        let steps = substitution_steps(&e, &|n| Some(if n == "a" { 2.0 } else { 3.0 }));
        let m = expr::expr_math(&steps[1].expr, steps[1].focus.as_deref());
        assert_eq!(m.text(), "6 + 3");
        let hot: Vec<String> = m
            .tones()
            .filter(|(_, t)| *t == Some(Tone::Trace))
            .map(|(p, _)| match p {
                Piece::Text(s) | Piece::Sub(s) | Piece::Sup(s) => s.clone(),
            })
            .collect();
        assert_eq!(hot, ["6"]);
    }

    #[test]
    fn plug_in_lists_each_formula_then_its_working() {
        assert!(plug_in(&[]).is_none());
        let d = plug_in(&[Substitution::new(
            MathText::new().t("y"),
            "2x + 1",
            &[("x", 3.0)],
        )])
        .unwrap();
        let texts: Vec<String> = d.steps.iter().map(|s| s.math.text()).collect();
        assert_eq!(texts, ["y = 2x + 1", "y = 2 · 3 + 1", "y = 6 + 1", "y = 7"]);
    }

    #[test]
    fn picks_and_steps_stay_in_range() {
        assert_eq!(clamp_steps(3, 9, &[]), (0, 0));
        assert_eq!(clamp_steps(0, 9, &[4]), (0, 3));
        assert_eq!(clamp_steps(5, 1, &[4, 2]), (1, 1));
        assert_eq!(clamp_steps(1, 0, &[4, 0]), (1, 0));
        assert_eq!(clamp_steps(0, 2, &[7, 2]), (0, 2));
    }
}
