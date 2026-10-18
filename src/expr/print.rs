//! Canonical, re-parseable text for an expression.

use super::ast::{BinOp, Expr};
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
