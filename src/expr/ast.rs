//! The expression tree and the definition forms the parser recognises.

use std::collections::BTreeSet;
use std::ops::Range;

/// An expression. The parser never folds literals: `-2` is `Neg(Num(2.0))`.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    /// A canonical variable name: Greek words become glyphs, subscripts stay (`k_r`).
    Var(String),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Call(Func, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sqrt,
    Abs,
    Exp,
    Ln,
    Log,
    Floor,
    Ceil,
    Min,
    Max,
}

impl Func {
    pub const ALL: [Func; 15] = [
        Func::Sin,
        Func::Cos,
        Func::Tan,
        Func::Asin,
        Func::Acos,
        Func::Atan,
        Func::Sqrt,
        Func::Abs,
        Func::Exp,
        Func::Ln,
        Func::Log,
        Func::Floor,
        Func::Ceil,
        Func::Min,
        Func::Max,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Asin => "asin",
            Func::Acos => "acos",
            Func::Atan => "atan",
            Func::Sqrt => "sqrt",
            Func::Abs => "abs",
            Func::Exp => "exp",
            Func::Ln => "ln",
            Func::Log => "log",
            Func::Floor => "floor",
            Func::Ceil => "ceil",
            Func::Min => "min",
            Func::Max => "max",
        }
    }

    /// Number of arguments: 2 for `min` and `max`, 1 for everything else.
    pub fn arity(self) -> usize {
        match self {
            Func::Min | Func::Max => 2,
            _ => 1,
        }
    }

    pub fn from_name(name: &str) -> Option<Func> {
        Func::ALL.into_iter().find(|f| f.name() == name)
    }
}

/// What a line of input defines.
#[derive(Clone, Debug, PartialEq)]
pub enum Definition {
    /// `y = ...`, `f(x) = ...`, or a bare expression (variable `"x"`).
    Explicit { var: String, body: Expr },
    /// `(x(t), y(t))`; `t` is `"t"`.
    Parametric { t: String, x: Expr, y: Expr },
    /// `r = ...`; `theta` is `"θ"` (input `theta` or `θ`).
    Polar { theta: String, r: Expr },
}

/// A parse failure with a byte span into the source.
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub span: Range<usize>,
    pub message: String,
}

/// Free parameters of a definition: sorted, deduplicated, excluding the
/// independent variable and the constants `π` and `e`.
pub fn free_params(def: &Definition) -> Vec<String> {
    fn walk(e: &Expr, skip: &str, out: &mut BTreeSet<String>) {
        match e {
            Expr::Num(_) => {}
            Expr::Var(n) => {
                if n != skip && n != "π" && n != "e" {
                    out.insert(n.clone());
                }
            }
            Expr::Neg(a) => walk(a, skip, out),
            Expr::Bin(_, a, b) => {
                walk(a, skip, out);
                walk(b, skip, out);
            }
            Expr::Call(_, args) => args.iter().for_each(|a| walk(a, skip, out)),
        }
    }
    let mut out = BTreeSet::new();
    match def {
        Definition::Explicit { var, body } => walk(body, var, &mut out),
        Definition::Parametric { t, x, y } => {
            walk(x, t, &mut out);
            walk(y, t, &mut out);
        }
        Definition::Polar { theta, r } => walk(r, theta, &mut out),
    }
    out.into_iter().collect()
}
