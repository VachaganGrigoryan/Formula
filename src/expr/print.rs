//! Canonical, re-parseable text for an expression.

use super::ast::{BinOp, Definition, Expr};
use crate::formula::MathText;
use std::fmt::Write;

/// Prints `e` with spaces around binary operators and the parentheses needed
/// for `parse_expr(&print(e)) == e`, e.g. `a * sin(b * x)`, `-(x ^ 2)`.
pub fn print(e: &Expr) -> String {
    let mut out = String::new();
    write_expr(e, &mut out);
    out
}

/// Binding strength: sums 1, products 2, negation 3, powers 4, atoms 5.
fn prec(e: &Expr) -> u8 {
    match e {
        Expr::Bin(BinOp::Add | BinOp::Sub, ..) => 1,
        Expr::Bin(BinOp::Mul | BinOp::Div, ..) => 2,
        Expr::Neg(_) => 3,
        Expr::Bin(BinOp::Pow, ..) => 4,
        Expr::Num(_) | Expr::Var(_) | Expr::Call(..) => 5,
    }
}

fn child(e: &Expr, parens: bool, out: &mut String) {
    if parens {
        out.push('(');
    }
    write_expr(e, out);
    if parens {
        out.push(')');
    }
}

fn write_expr(e: &Expr, out: &mut String) {
    match e {
        Expr::Num(v) => {
            let _ = write!(out, "{v}");
        }
        Expr::Var(n) => out.push_str(n),
        Expr::Neg(a) => {
            out.push('-');
            child(a, prec(a) < 5, out);
        }
        Expr::Bin(BinOp::Pow, l, r) => {
            child(l, prec(l) < 5, out);
            out.push_str(" ^ ");
            child(r, prec(r) < 5, out);
        }
        Expr::Bin(op, l, r) => {
            let p = prec(e);
            child(l, prec(l) < p, out);
            out.push_str(match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Pow => unreachable!(),
            });
            // Left-associative: an equal-precedence right operand needs parentheses.
            child(r, prec(r) <= p, out);
        }
        Expr::Call(f, args) => {
            out.push_str(f.name());
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_expr(a, out);
            }
            out.push(')');
        }
    }
}

/// One run of typeset text: plain, subscript or superscript.
enum Run {
    T(String),
    Sub(String),
    Sup(String),
}

/// The definition typeset for the Inspector and LaTeX export: implicit
/// products as spaces, powers as superscripts, subscripted names (`k_r`).
pub fn to_math(def: &Definition) -> MathText {
    let mut runs = Vec::new();
    match def {
        Definition::Explicit { var, body } => {
            let head = if var == "x" {
                "y = ".to_string()
            } else {
                format!("f({var}) = ")
            };
            runs.push(Run::T(head));
            math(body, &mut runs);
        }
        Definition::Polar { r, .. } => {
            runs.push(Run::T("r = ".into()));
            math(r, &mut runs);
        }
        Definition::Parametric { x, y, .. } => {
            runs.push(Run::T("(".into()));
            math(x, &mut runs);
            runs.push(Run::T(", ".into()));
            math(y, &mut runs);
            runs.push(Run::T(")".into()));
        }
    }
    runs.into_iter().fold(MathText::new(), |m, r| match r {
        Run::T(s) => m.t(&s),
        Run::Sub(s) => m.sub(&s),
        Run::Sup(s) => m.sup(&s),
    })
}

fn math_child(e: &Expr, parens: bool, runs: &mut Vec<Run>) {
    if parens {
        runs.push(Run::T("(".into()));
    }
    math(e, runs);
    if parens {
        runs.push(Run::T(")".into()));
    }
}

fn math(e: &Expr, runs: &mut Vec<Run>) {
    match e {
        Expr::Num(v) => runs.push(Run::T(format!("{v}"))),
        Expr::Var(n) => match n.split_once('_') {
            Some((base, sub)) => {
                runs.push(Run::T(base.into()));
                runs.push(Run::Sub(sub.into()));
            }
            None => runs.push(Run::T(n.clone())),
        },
        Expr::Neg(a) => {
            runs.push(Run::T("-".into()));
            math_child(a, prec(a) < 4, runs);
        }
        Expr::Bin(BinOp::Pow, base, exp) => {
            math_child(base, prec(base) < 5, runs);
            // An exponent is one superscript run, so nested powers stay as `^`.
            runs.push(Run::Sup(print(exp)));
        }
        Expr::Bin(op, l, r) => {
            let p = prec(e);
            math_child(l, prec(l) < p, runs);
            let sep = match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Div => " / ",
                // `2x`, `a b`, but `x · 2` so two numbers never run together.
                BinOp::Mul => match (&**l, &**r) {
                    (_, Expr::Num(_)) => " · ",
                    (Expr::Num(_), Expr::Var(_)) => "",
                    _ => " ",
                },
                BinOp::Pow => unreachable!(),
            };
            runs.push(Run::T(sep.into()));
            math_child(r, prec(r) <= p, runs);
        }
        Expr::Call(f, args) => {
            runs.push(Run::T(format!("{}(", f.name())));
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    runs.push(Run::T(", ".into()));
                }
                math(a, runs);
            }
            runs.push(Run::T(")".into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::parse;
    use crate::formula::Piece;

    fn text(src: &str) -> String {
        to_math(&parse(src).unwrap()).text()
    }

    #[test]
    fn typesets_definitions() {
        assert_eq!(text("y = a sin(bx)"), "y = a sin(b x)");
        assert_eq!(text("f(t) = 2t + 1"), "f(t) = 2t + 1");
        assert_eq!(text("r = 1 + cos(3θ)"), "r = 1 + cos(3θ)");
        assert_eq!(text("(cos t, sin t)"), "(cos(t), sin(t))");
        assert_eq!(text("x*2"), "y = x · 2");
        assert_eq!(text("-(x+1)/2"), "y = -(x + 1) / 2");
    }

    #[test]
    fn powers_and_subscripts_are_raised_and_lowered() {
        let m = to_math(&parse("y = k_r x^2").unwrap());
        let pieces: Vec<String> = m
            .pieces()
            .map(|p| match p {
                Piece::Text(s) => format!("T{s}"),
                Piece::Sub(s) => format!("_{s}"),
                Piece::Sup(s) => format!("^{s}"),
            })
            .collect();
        assert!(pieces.contains(&"_r".to_string()), "{pieces:?}");
        assert!(pieces.contains(&"^2".to_string()), "{pieces:?}");
    }

    #[test]
    fn converts_to_ascii_latex() {
        for src in [
            "y = a sin(bx)",
            "r = 1 + cos(kθ)",
            "(cos t, a sin t)",
            "y = e^(-x^2)",
        ] {
            let tex = crate::latex::latex_of(&to_math(&parse(src).unwrap()));
            assert!(tex.is_ascii(), "{src}: {tex}");
        }
        let tex = crate::latex::latex_of(&to_math(&parse("y = a sin(bx)").unwrap()));
        assert!(tex.contains("\\sin"), "{tex}");
    }
}
