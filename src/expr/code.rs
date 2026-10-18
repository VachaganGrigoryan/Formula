//! Printing an expression as Python (numpy), TypeScript or Rust source.

use super::ast::{BinOp, Expr, Func};
use crate::content::Lang;

/// ASCII names for Greek letters. `λ` is `lam` because `lambda` is a Python keyword.
const GREEK: [(char, &str); 25] = [
    ('α', "alpha"),
    ('β', "beta"),
    ('γ', "gamma"),
    ('δ', "delta"),
    ('ε', "epsilon"),
    ('ζ', "zeta"),
    ('η', "eta"),
    ('θ', "theta"),
    ('ι', "iota"),
    ('κ', "kappa"),
    ('λ', "lam"),
    ('μ', "mu"),
    ('ν', "nu"),
    ('ξ', "xi"),
    ('ο', "omicron"),
    ('π', "pi"),
    ('ρ', "rho"),
    ('ς', "sigmaf"),
    ('σ', "sigma"),
    ('τ', "tau"),
    ('υ', "upsilon"),
    ('φ', "phi"),
    ('χ', "chi"),
    ('ψ', "psi"),
    ('ω', "omega"),
];

/// A variable name as an identifier in `lang`: `θ` → `theta`, `k_r` stays,
/// and the constants `π` and `e` become the language's own.
pub fn ident(name: &str, lang: Lang) -> String {
    match (name, lang) {
        ("π", Lang::Python) => return "np.pi".into(),
        ("π", Lang::TypeScript) => return "Math.PI".into(),
        ("π", Lang::Rust) => return "std::f64::consts::PI".into(),
        ("e", Lang::Python) => return "np.e".into(),
        ("e", Lang::TypeScript) => return "Math.E".into(),
        ("e", Lang::Rust) => return "std::f64::consts::E".into(),
        _ => {}
    }
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii() {
            out.push(c);
        } else if let Some((_, w)) = GREEK.iter().find(|(g, _)| *g == c) {
            out.push_str(w);
        } else if let Some((_, w)) = c
            .to_lowercase()
            .next()
            .and_then(|l| GREEK.iter().find(|(g, _)| *g == l))
        {
            // Capital Greek: `Ω` → `Omega`.
            out.push_str(&w[..1].to_ascii_uppercase());
            out.push_str(&w[1..]);
        } else {
            out.push_str(&format!("u{:x}", c as u32));
        }
    }
    out
}

/// `e` as source code in `lang`, with parentheses only where precedence needs them.
pub fn to_code(e: &Expr, lang: Lang) -> String {
    let mut out = String::new();
    write(e, lang, &mut out);
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

fn wrapped(e: &Expr, lang: Lang, parens: bool, out: &mut String) {
    if parens {
        out.push('(');
    }
    write(e, lang, out);
    if parens {
        out.push(')');
    }
}

fn number(v: f64, lang: Lang) -> String {
    let s = format!("{v}");
    if lang == Lang::Rust && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

/// A Rust method call `recv.method(arg)`. A literal receiver gets a type
/// suffix and a receiver with no variable in it uses `f64::method(recv, arg)`,
/// since `(-2.0).sin()` does not compile (the float type is ambiguous).
fn method_call(recv: &Expr, method: &str, arg: Option<&Expr>, out: &mut String) {
    match recv {
        Expr::Num(v) => {
            out.push_str(&number(*v, Lang::Rust));
            out.push_str("_f64");
        }
        Expr::Var(_) | Expr::Call(..) if has_var(recv) => write(recv, Lang::Rust, out),
        _ if has_var(recv) => wrapped(recv, Lang::Rust, true, out),
        _ => {
            out.push_str("f64::");
            out.push_str(method);
            out.push('(');
            write(recv, Lang::Rust, out);
            if let Some(b) = arg {
                out.push_str(", ");
                write(b, Lang::Rust, out);
            }
            out.push(')');
            return;
        }
    }
    out.push('.');
    out.push_str(method);
    out.push('(');
    if let Some(b) = arg {
        write(b, Lang::Rust, out);
    }
    out.push(')');
}

/// Whether `e` mentions a variable or constant, which fixes its type as `f64`.
fn has_var(e: &Expr) -> bool {
    match e {
        Expr::Num(_) => false,
        Expr::Var(_) => true,
        Expr::Neg(a) => has_var(a),
        Expr::Bin(_, a, b) => has_var(a) || has_var(b),
        Expr::Call(_, args) => args.iter().any(has_var),
    }
}

fn write(e: &Expr, lang: Lang, out: &mut String) {
    match e {
        Expr::Num(v) => out.push_str(&number(*v, lang)),
        Expr::Var(n) => out.push_str(&ident(n, lang)),
        Expr::Neg(a) => {
            out.push('-');
            // `--x` is a decrement in TypeScript, so a nested negation keeps its parentheses.
            wrapped(a, lang, prec(a) < 3 || matches!(**a, Expr::Neg(_)), out);
        }
        Expr::Bin(BinOp::Pow, a, b) => match lang {
            Lang::Python => {
                wrapped(a, lang, prec(a) < 5, out);
                out.push_str(" ** ");
                wrapped(b, lang, prec(b) < 3, out);
            }
            Lang::TypeScript => {
                out.push_str("Math.pow(");
                write(a, lang, out);
                out.push_str(", ");
                write(b, lang, out);
                out.push(')');
            }
            Lang::Rust => {
                method_call(a, "powf", Some(b), out);
            }
        },
        Expr::Bin(op, a, b) => {
            let p = prec(e);
            wrapped(a, lang, prec(a) < p, out);
            out.push_str(match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Pow => unreachable!(),
            });
            wrapped(b, lang, prec(b) <= p, out);
        }
        Expr::Call(f, args) => match lang {
            Lang::Rust => {
                method_call(&args[0], rust_method(*f), args.get(1), out);
            }
            _ => {
                out.push_str(function(*f, lang));
                out.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write(a, lang, out);
                }
                out.push(')');
            }
        },
    }
}

fn function(f: Func, lang: Lang) -> &'static str {
    let py = lang == Lang::Python;
    match f {
        Func::Sin => {
            if py {
                "np.sin"
            } else {
                "Math.sin"
            }
        }
        Func::Cos => {
            if py {
                "np.cos"
            } else {
                "Math.cos"
            }
        }
        Func::Tan => {
            if py {
                "np.tan"
            } else {
                "Math.tan"
            }
        }
        Func::Asin => {
            if py {
                "np.arcsin"
            } else {
                "Math.asin"
            }
        }
        Func::Acos => {
            if py {
                "np.arccos"
            } else {
                "Math.acos"
            }
        }
        Func::Atan => {
            if py {
                "np.arctan"
            } else {
                "Math.atan"
            }
        }
        Func::Sqrt => {
            if py {
                "np.sqrt"
            } else {
                "Math.sqrt"
            }
        }
        Func::Abs => {
            if py {
                "np.abs"
            } else {
                "Math.abs"
            }
        }
        Func::Exp => {
            if py {
                "np.exp"
            } else {
                "Math.exp"
            }
        }
        Func::Ln => {
            if py {
                "np.log"
            } else {
                "Math.log"
            }
        }
        Func::Log => {
            if py {
                "np.log10"
            } else {
                "Math.log10"
            }
        }
        Func::Floor => {
            if py {
                "np.floor"
            } else {
                "Math.floor"
            }
        }
        Func::Ceil => {
            if py {
                "np.ceil"
            } else {
                "Math.ceil"
            }
        }
        Func::Min => {
            if py {
                "np.minimum"
            } else {
                "Math.min"
            }
        }
        Func::Max => {
            if py {
                "np.maximum"
            } else {
                "Math.max"
            }
        }
    }
}

fn rust_method(f: Func) -> &'static str {
    match f {
        Func::Ln => "ln",
        Func::Log => "log10",
        _ => f.name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{parse_expr, random_expr};
    use crate::rng::Rng;

    fn code(src: &str) -> [String; 3] {
        let e = parse_expr(src).unwrap();
        Lang::ALL.map(|l| to_code(&e, l))
    }

    #[test]
    fn calls_in_every_language() {
        assert_eq!(
            code("sin(b x)"),
            ["np.sin(b * x)", "Math.sin(b * x)", "(b * x).sin()"]
        );
        assert_eq!(code("-x^2"), ["-x ** 2", "-Math.pow(x, 2)", "-x.powf(2.0)"]);
        assert_eq!(code("2^x"), ["2 ** x", "Math.pow(2, x)", "2.0_f64.powf(x)"]);
        assert_eq!(
            code("max(a, θ)"),
            ["np.maximum(a, theta)", "Math.max(a, theta)", "a.max(theta)"]
        );
        assert_eq!(
            code("e^x"),
            [
                "np.e ** x",
                "Math.pow(Math.E, x)",
                "std::f64::consts::E.powf(x)"
            ]
        );
    }

    #[test]
    fn parentheses_follow_precedence() {
        assert_eq!(code("(a + b) c")[0], "(a + b) * c");
        assert_eq!(code("a - (b - c)")[0], "a - (b - c)");
        assert_eq!(code("(-x)^2")[0], "(-x) ** 2");
        assert_eq!(code("2^-x")[0], "2 ** -x");
        assert_eq!(code("2^(x+1)")[0], "2 ** (x + 1)");
        assert_eq!(code("--x")[1], "-(-x)");
        assert_eq!(code("(x + 1)^2")[2], "(x + 1.0).powf(2.0)");
        assert_eq!(code("sqrt(2)")[2], "2.0_f64.sqrt()");
        assert_eq!(code("ln x + log x")[2], "x.ln() + x.log10()");
        assert_eq!(code("sin(-2)")[2], "f64::sin(-2.0)");
        assert_eq!(code("(1 + 2)^x")[2], "f64::powf(1.0 + 2.0, x)");
        assert_eq!(code("max(1, 2)")[2], "1.0_f64.max(2.0)");
        assert_eq!(code("sin(pi/2)")[2], "(std::f64::consts::PI / 2.0).sin()");
    }

    #[test]
    fn identifiers_are_ascii() {
        assert_eq!(ident("θ", Lang::Python), "theta");
        assert_eq!(ident("θ_1", Lang::Rust), "theta_1");
        assert_eq!(ident("k_r", Lang::TypeScript), "k_r");
        assert_eq!(ident("λ", Lang::Python), "lam");
        assert_eq!(ident("Ω", Lang::Python), "Omega");
        assert_eq!(ident("π", Lang::Rust), "std::f64::consts::PI");
    }

    #[test]
    fn random_expressions_give_ascii_balanced_code() {
        let mut rng = Rng::new(11);
        for _ in 0..300 {
            let e = random_expr(&mut rng, 5);
            for lang in Lang::ALL {
                let s = to_code(&e, lang);
                assert!(s.is_ascii(), "{s}");
                let mut depth = 0i32;
                for c in s.chars() {
                    depth += match c {
                        '(' => 1,
                        ')' => -1,
                        _ => 0,
                    };
                    assert!(depth >= 0, "{s}");
                }
                assert_eq!(depth, 0, "{s}");
                assert!(!s.contains("--"), "{s}");
            }
        }
    }
}
