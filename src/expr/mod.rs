//! A small in-repo expression engine: lexer, AST, Pratt parser, evaluator,
//! and printers for text, typeset math ([`to_math`]) and code ([`to_code`]).
//!
//! Apart from building a `MathText`, this module does not touch egui, so it is
//! unit-tested on its own and shared by the `expression` tool and its exports.
//!
//! Variables follow the single-letter convention of Desmos: a run of letters
//! splits into one variable per letter (`ab` is `a * b`), except for known
//! words (function names, Greek names and the constants `pi` and `e`), which
//! stay whole. At each position the longest known word wins, so `pix` is
//! `π * x` and `sinx` is `sin` applied to `x`.

mod ast;
mod code;
mod eval;
mod lexer;
mod parser;
mod print;

pub use ast::{BinOp, Definition, Expr, Func, ParseError, free_params};
pub use code::{ident, to_code};
pub use eval::{constant, eval};
pub use parser::{parse, parse_expr};
pub use print::{expr_math, number, print, to_math};

/// A seeded random AST that [`parse_expr`] accepts: positive literals, only
/// parseable names, and calls with the right arity. Shared with later tests.
#[cfg(test)]
pub(crate) fn random_expr(rng: &mut crate::rng::Rng, depth: u32) -> Expr {
    const NAMES: [&str; 10] = ["x", "t", "a", "b", "w", "k_r", "a_1", "θ", "π", "e"];
    const OPS: [BinOp; 5] = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div, BinOp::Pow];
    if depth == 0 || rng.index(4) == 0 {
        return if rng.index(2) == 0 {
            let scale = [1.0, 0.1, 0.01, 100.0][rng.index(4)];
            Expr::Num((rng.index(10_000) + 1) as f64 * scale)
        } else {
            Expr::Var(NAMES[rng.index(NAMES.len())].to_string())
        };
    }
    match rng.index(6) {
        0 => Expr::Neg(Box::new(random_expr(rng, depth - 1))),
        1 => {
            let f = Func::ALL[rng.index(Func::ALL.len())];
            Expr::Call(
                f,
                (0..f.arity())
                    .map(|_| random_expr(rng, depth - 1))
                    .collect(),
            )
        }
        _ => {
            let op = OPS[rng.index(OPS.len())];
            let l = random_expr(rng, depth - 1);
            let r = random_expr(rng, depth - 1);
            Expr::Bin(op, Box::new(l), Box::new(r))
        }
    }
}
