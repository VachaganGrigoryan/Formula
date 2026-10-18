//! Evaluating an expression with `f64` semantics: `1/0` is infinite and
//! `ln(-1)` is NaN, so callers decide what to skip.

use super::ast::{BinOp, Expr, Func};

/// The value of a built-in constant (`π`, `e`), if `name` is one.
pub fn constant(name: &str) -> Option<f64> {
    match name {
        "π" => Some(std::f64::consts::PI),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}

/// Evaluates `e`, looking variables up in `env` first and then in the
/// constants. An unknown variable makes the result NaN.
pub fn eval(e: &Expr, env: &dyn Fn(&str) -> Option<f64>) -> f64 {
    match e {
        Expr::Num(v) => *v,
        Expr::Var(n) => env(n).or_else(|| constant(n)).unwrap_or(f64::NAN),
        Expr::Neg(a) => -eval(a, env),
        Expr::Bin(op, a, b) => {
            let (a, b) = (eval(a, env), eval(b, env));
            match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => a / b,
                BinOp::Pow => a.powf(b),
            }
        }
        Expr::Call(f, args) => {
            let x = args.first().map_or(f64::NAN, |a| eval(a, env));
            let y = || args.get(1).map_or(f64::NAN, |a| eval(a, env));
            match f {
                Func::Sin => x.sin(),
                Func::Cos => x.cos(),
                Func::Tan => x.tan(),
                Func::Asin => x.asin(),
                Func::Acos => x.acos(),
                Func::Atan => x.atan(),
                Func::Sqrt => x.sqrt(),
                Func::Abs => x.abs(),
                Func::Exp => x.exp(),
                Func::Ln => x.ln(),
                Func::Log => x.log10(),
                Func::Floor => x.floor(),
                Func::Ceil => x.ceil(),
                Func::Min => x.min(y()),
                Func::Max => x.max(y()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::parse_expr;

    fn val(src: &str) -> f64 {
        eval(&parse_expr(src).unwrap(), &|n| (n == "x").then_some(0.5))
    }

    #[test]
    fn follows_f64_semantics() {
        assert_eq!(val("1/0"), f64::INFINITY);
        assert!(val("ln(-1)").is_nan());
        assert!(val("sqrt(-1)").is_nan());
        assert_eq!(val("2^3^2"), 512.0);
        assert_eq!(val("-2^2"), -4.0);
        assert!((val("pi") - std::f64::consts::PI).abs() < 1e-15);
        assert!((val("e") - std::f64::consts::E).abs() < 1e-15);
        assert!(val("q").is_nan(), "unknown variable");
        assert_eq!(val("2x"), 1.0);
    }

    #[test]
    fn variables_shadow_constants() {
        let e = parse_expr("e").unwrap();
        assert_eq!(eval(&e, &|n| (n == "e").then_some(3.0)), 3.0);
    }

    #[test]
    fn every_function_matches_its_f64_method() {
        let x = 0.3_f64;
        for f in Func::ALL {
            let want = match f {
                Func::Sin => x.sin(),
                Func::Cos => x.cos(),
                Func::Tan => x.tan(),
                Func::Asin => x.asin(),
                Func::Acos => x.acos(),
                Func::Atan => x.atan(),
                Func::Sqrt => x.sqrt(),
                Func::Abs => x.abs(),
                Func::Exp => x.exp(),
                Func::Ln => x.ln(),
                Func::Log => x.log10(),
                Func::Floor => x.floor(),
                Func::Ceil => x.ceil(),
                Func::Min => x.min(2.0),
                Func::Max => x.max(2.0),
            };
            let args = (0..f.arity()).map(|i| Expr::Num([x, 2.0][i])).collect();
            assert_eq!(eval(&Expr::Call(f, args), &|_| None), want, "{}", f.name());
        }
    }
}
