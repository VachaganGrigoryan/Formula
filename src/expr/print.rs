//! Canonical, re-parseable text for an expression.

use super::ast::{BinOp, Definition, Expr};
use crate::formula::{MathText, Tone};
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

/// Typesets expressions into runs, optionally rounding numbers and
/// highlighting the subtree at one path (child indices from the root).
struct Typeset<'a> {
    runs: Vec<(Run, bool)>,
    round: bool,
    focus: Option<&'a [usize]>,
}

impl Typeset<'_> {
    fn push(&mut self, run: Run, hot: bool) {
        self.runs.push((run, hot));
    }

    fn text(&mut self, s: &str, hot: bool) {
        self.push(Run::T(s.into()), hot);
    }

    fn finish(self) -> MathText {
        self.runs.into_iter().fold(MathText::new(), |m, (r, hot)| {
            let m = match r {
                Run::T(s) => m.t(&s),
                Run::Sub(s) => m.sub(&s),
                Run::Sup(s) => m.sup(&s),
            };
            if hot { m.tone(Tone::Trace) } else { m }
        })
    }

    fn number(&self, v: f64) -> String {
        if self.round {
            number(v)
        } else {
            format!("{v}")
        }
    }

    /// `e` at `path`. `hot` is set inside the focused subtree; `wrap_negative`
    /// puts a negative literal in parentheses (`3 - (-2)`, `(-2)²`).
    fn expr(&mut self, e: &Expr, path: &mut Vec<usize>, hot: bool, wrap_negative: bool) {
        let hot = hot || self.focus == Some(path.as_slice());
        match e {
            Expr::Num(v) => {
                let s = self.number(*v);
                if wrap_negative && *v < 0.0 {
                    self.text(&format!("({s})"), hot);
                } else {
                    self.text(&s, hot);
                }
            }
            Expr::Var(n) => match n.split_once('_') {
                Some((base, sub)) => {
                    self.push(Run::T(base.into()), hot);
                    self.push(Run::Sub(sub.into()), hot);
                }
                None => self.text(n, hot),
            },
            Expr::Neg(a) => {
                self.text("-", hot);
                self.child(a, 0, prec(a) < 4, true, path, hot);
            }
            Expr::Bin(BinOp::Pow, base, exp) => {
                self.child(base, 0, prec(base) < 5, true, path, hot);
                // An exponent is one superscript run, so nested powers stay as `^`.
                let mut inner = Typeset {
                    runs: Vec::new(),
                    round: self.round,
                    focus: None,
                };
                path.push(1);
                let exp_hot = hot || self.focus.is_some_and(|f| f.starts_with(path));
                inner.expr(exp, path, false, false);
                path.pop();
                let text: String = inner
                    .runs
                    .into_iter()
                    .map(|(r, _)| match r {
                        Run::T(s) | Run::Sub(s) | Run::Sup(s) => s,
                    })
                    .collect();
                self.push(Run::Sup(text.replace(' ', "")), exp_hot);
            }
            Expr::Bin(op, l, r) => {
                let p = prec(e);
                self.child(l, 0, prec(l) < p, false, path, hot);
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
                self.text(sep, hot);
                self.child(r, 1, prec(r) <= p, true, path, hot);
            }
            Expr::Call(f, args) => {
                self.text(&format!("{}(", f.name()), hot);
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.text(", ", hot);
                    }
                    path.push(i);
                    self.expr(a, path, hot, false);
                    path.pop();
                }
                self.text(")", hot);
            }
        }
    }

    fn child(
        &mut self,
        e: &Expr,
        index: usize,
        parens: bool,
        wrap_negative: bool,
        path: &mut Vec<usize>,
        hot: bool,
    ) {
        path.push(index);
        let hot_here = hot || self.focus == Some(path.as_slice());
        if parens {
            self.text("(", hot_here);
        }
        self.expr(e, path, hot, wrap_negative && !parens);
        if parens {
            self.text(")", hot_here);
        }
        path.pop();
    }
}

/// A number for a worked step: at most four significant digits, scientific
/// notation when very large or small, and words for what is not a number.
pub fn number(v: f64) -> String {
    if v.is_nan() {
        return "undefined".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "∞".into() } else { "-∞".into() };
    }
    if v == 0.0 {
        return "0".into();
    }
    let trim = |s: String| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };
    if !(1e-4..1e6).contains(&v.abs()) {
        let s = format!("{v:.3e}");
        let (mantissa, exponent) = s.split_once('e').unwrap_or((&s, "0"));
        return format!("{}e{exponent}", trim(mantissa.to_string()));
    }
    let digits = v.abs().log10().floor() as i32 + 1;
    let decimals = (4 - digits).max(0) as usize;
    trim(format!("{v:.decimals$}"))
}

/// The definition typeset for the Inspector and LaTeX export: implicit
/// products as spaces, powers as superscripts, subscripted names (`k_r`).
pub fn to_math(def: &Definition) -> MathText {
    let mut t = Typeset {
        runs: Vec::new(),
        round: false,
        focus: None,
    };
    let mut path = Vec::new();
    match def {
        Definition::Explicit { var, body } => {
            let head = if var == "x" {
                "y = ".to_string()
            } else {
                format!("f({var}) = ")
            };
            t.text(&head, false);
            t.expr(body, &mut path, false, false);
        }
        Definition::Polar { r, .. } => {
            t.text("r = ", false);
            t.expr(r, &mut path, false, false);
        }
        Definition::Parametric { x, y, .. } => {
            t.text("(", false);
            t.expr(x, &mut path, false, false);
            t.text(", ", false);
            t.expr(y, &mut path, false, false);
            t.text(")", false);
        }
    }
    t.finish()
}

/// One expression typeset for a worked step: numbers rounded by [`number`]
/// and the subtree at `focus` (child indices from the root) in the trace tone.
pub fn expr_math(e: &Expr, focus: Option<&[usize]>) -> MathText {
    let mut t = Typeset {
        runs: Vec::new(),
        round: true,
        focus,
    };
    t.expr(e, &mut Vec::new(), false, false);
    t.finish()
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
