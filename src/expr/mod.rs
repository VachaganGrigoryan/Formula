//! A small in-repo expression engine: lexer, AST, Pratt parser and printer.
//!
//! This module is UI-free (no egui) so it can be unit-tested and reused by
//! the evaluator, the exporters and the `expression` tool.
//!
//! Variables follow the single-letter convention of Desmos: a run of letters
//! splits into one variable per letter (`ab` is `a * b`), except for known
//! words (function names, Greek names and the constants `pi` and `e`), which
//! stay whole. At each position the longest known word wins, so `pix` is
//! `π * x` and `sinx` is `sin` applied to `x`.

mod ast;
mod lexer;
mod parser;
mod print;

pub use ast::{BinOp, Definition, Expr, Func, ParseError, free_params};
pub use parser::{parse, parse_expr};
pub use print::print;

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
