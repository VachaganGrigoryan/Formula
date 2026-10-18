//! A Pratt-style recursive-descent parser for expressions and definitions.
//!
//! Precedence, lowest to highest: `+ -`; `* /` and implicit multiplication
//! (same level, left-associative); unary `-`; `^` (right-associative). A
//! function without parentheses takes the implicit product that follows it.

use super::ast::{BinOp, Definition, Expr, Func, ParseError};
use super::lexer::{Tok, Token, lex};
use std::ops::Range;

/// Maximum nesting (parentheses, negations, powers, long operator chains).
const MAX_DEPTH: usize = 256;

const HEAD_HELP: &str = "Write y = ..., r = ..., f(x) = ... or (x(t), y(t))";

type Res<T> = Result<T, ParseError>;

fn err(span: Range<usize>, message: impl Into<String>) -> ParseError {
    ParseError {
        span,
        message: message.into(),
    }
}

/// Parses a line of input into what it defines.
pub fn parse(src: &str) -> Res<Definition> {
    let toks = lex(src)?;
    if toks.is_empty() {
        return Err(empty(src));
    }
    let mut p = Parser::new(toks, src.len());

    if let Some(eq) = p.top_level_eq() {
        let head = p.head(eq)?;
        p.pos = eq + 1;
        let body = p.sum()?;
        p.finish()?;
        return Ok(match head {
            Head::Y => Definition::Explicit {
                var: "x".into(),
                body,
            },
            Head::Polar => Definition::Polar {
                theta: "θ".into(),
                r: body,
            },
            Head::Func(var) => Definition::Explicit { var, body },
        });
    }

    if p.is_pair() {
        p.pos = 1;
        let x = p.sum()?;
        p.expect(&Tok::Comma, "','")?;
        let y = p.sum()?;
        p.expect(&Tok::RParen, "')'")?;
        p.finish()?;
        return Ok(Definition::Parametric {
            t: "t".into(),
            x,
            y,
        });
    }

    let body = p.sum()?;
    p.finish()?;
    Ok(Definition::Explicit {
        var: "x".into(),
        body,
    })
}

/// Parses a body expression only.
pub fn parse_expr(src: &str) -> Res<Expr> {
    let toks = lex(src)?;
    if toks.is_empty() {
        return Err(empty(src));
    }
    let mut p = Parser::new(toks, src.len());
    let e = p.sum()?;
    p.finish()?;
    Ok(e)
}

fn empty(src: &str) -> ParseError {
    err(src.len()..src.len(), "type an expression")
}

enum Head {
    Y,
    Polar,
    Func(String),
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    depth: usize,
    end: usize,
}

impl Parser {
    fn new(toks: Vec<Token>, end: usize) -> Self {
        Self {
            toks,
            pos: 0,
            depth: 0,
            end,
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.kind)
    }

    /// The span of the current token, or the end of the input.
    fn here(&self) -> Range<usize> {
        self.toks
            .get(self.pos)
            .map_or(self.end..self.end, |t| t.span.clone())
    }

    /// The end of the last consumed token.
    fn prev_end(&self) -> usize {
        self.pos.checked_sub(1).map_or(0, |i| self.toks[i].span.end)
    }

    fn unexpected(&self) -> ParseError {
        let what = match self.peek() {
            None => return err(self.here(), "unexpected end of input"),
            Some(Tok::Num(_)) => "number".to_string(),
            Some(Tok::Ident(n)) => format!("'{n}'"),
            Some(Tok::Plus) => "'+'".into(),
            Some(Tok::Minus) => "'-'".into(),
            Some(Tok::Star) => "'*'".into(),
            Some(Tok::Slash) => "'/'".into(),
            Some(Tok::Caret) => "'^'".into(),
            Some(Tok::LParen) => "'('".into(),
            Some(Tok::RParen) => "')'".into(),
            Some(Tok::Comma) => "','".into(),
            Some(Tok::Eq) => "'='".into(),
        };
        err(self.here(), format!("unexpected {what}"))
    }

    fn expect(&mut self, tok: &Tok, what: &str) -> Res<()> {
        if self.peek() == Some(tok) {
            self.pos += 1;
            Ok(())
        } else if self.peek().is_none() {
            Err(err(self.here(), format!("expected {what}")))
        } else {
            let mut e = self.unexpected();
            e.message = format!("{}, expected {what}", e.message);
            Err(e)
        }
    }

    /// Errors unless every token was consumed.
    fn finish(&self) -> Res<()> {
        match self.peek() {
            None => Ok(()),
            Some(Tok::Num(_)) => Err(err(self.here(), "missing operator")),
            Some(_) => Err(self.unexpected()),
        }
    }

    /// Spends one level of the depth budget.
    fn bump(&mut self) -> Res<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(err(self.here(), "too deeply nested"));
        }
        Ok(())
    }

    /// Runs `f` one level deeper.
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Res<T>) -> Res<T> {
        let base = self.depth;
        self.bump()?;
        let out = f(self);
        self.depth = base;
        out
    }

    /// The index of the first `=` outside any parentheses.
    fn top_level_eq(&self) -> Option<usize> {
        let mut level = 0usize;
        for (i, t) in self.toks.iter().enumerate() {
            match t.kind {
                Tok::LParen => level += 1,
                Tok::RParen => level = level.saturating_sub(1),
                Tok::Eq if level == 0 => return Some(i),
                _ => {}
            }
        }
        None
    }

    /// Whether the input is `( ... , ... )` as a whole.
    fn is_pair(&self) -> bool {
        if self.toks.first().map(|t| &t.kind) != Some(&Tok::LParen) {
            return false;
        }
        let mut level = 0usize;
        let mut comma = false;
        for (i, t) in self.toks.iter().enumerate() {
            match t.kind {
                Tok::LParen => level += 1,
                Tok::RParen => {
                    level -= 1;
                    if level == 0 {
                        return comma && i == self.toks.len() - 1;
                    }
                }
                Tok::Comma if level == 1 => comma = true,
                _ => {}
            }
        }
        false
    }

    /// Classifies the tokens before the `=` at index `eq`.
    fn head(&self, eq: usize) -> Res<Head> {
        let bad = || err(self.toks[0].span.start..self.toks[eq].span.end, HEAD_HELP);
        let kinds: Vec<&Tok> = self.toks[..eq].iter().map(|t| &t.kind).collect();
        match kinds.as_slice() {
            [Tok::Ident(n)] if n == "y" => Ok(Head::Y),
            [Tok::Ident(n)] if n == "r" => Ok(Head::Polar),
            [Tok::Ident(f), Tok::LParen, Tok::Ident(v), Tok::RParen]
                if f.chars().count() == 1 && Func::from_name(v).is_none() =>
            {
                Ok(Head::Func(v.clone()))
            }
            _ => Err(bad()),
        }
    }

    /// `term (('+' | '-') term)*`
    fn sum(&mut self) -> Res<Expr> {
        let base = self.depth;
        let mut lhs = self.term()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.term()?;
            self.bump()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        self.depth = base;
        Ok(lhs)
    }

    /// `unary (('*' | '/' | implicit) unary)*`
    fn term(&mut self) -> Res<Expr> {
        let base = self.depth;
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => {
                    self.pos += 1;
                    BinOp::Mul
                }
                Some(Tok::Slash) => {
                    self.pos += 1;
                    BinOp::Div
                }
                Some(Tok::Ident(_) | Tok::LParen) => BinOp::Mul,
                _ => break,
            };
            let rhs = self.unary()?;
            self.bump()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        self.depth = base;
        Ok(lhs)
    }

    fn unary(&mut self) -> Res<Expr> {
        if self.peek() == Some(&Tok::Minus) {
            self.pos += 1;
            let operand = self.nested(Self::unary)?;
            return Ok(Expr::Neg(Box::new(operand)));
        }
        self.power()
    }

    /// `atom ('^' unary)?`, right-associative because the exponent is a unary.
    fn power(&mut self) -> Res<Expr> {
        let base = self.atom()?;
        if self.peek() == Some(&Tok::Caret) {
            self.pos += 1;
            let exp = self.nested(Self::unary)?;
            return Ok(Expr::Bin(BinOp::Pow, Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Res<Expr> {
        match self.peek() {
            Some(Tok::Num(v)) => {
                let v = *v;
                self.pos += 1;
                Ok(Expr::Num(v))
            }
            Some(Tok::Ident(name)) => {
                let name = name.clone();
                match Func::from_name(&name) {
                    Some(f) => self.nested(|p| p.call(f)),
                    None => {
                        self.pos += 1;
                        Ok(Expr::Var(name))
                    }
                }
            }
            Some(Tok::LParen) => self.nested(|p| {
                p.pos += 1;
                let e = p.sum()?;
                p.expect(&Tok::RParen, "')'")?;
                Ok(e)
            }),
            _ => Err(self.unexpected()),
        }
    }

    /// A call to `f`, with or without parentheses. The current token is the name.
    fn call(&mut self, f: Func) -> Res<Expr> {
        let start = self.here().start;
        self.pos += 1;
        let mut args = Vec::new();
        if self.peek() == Some(&Tok::LParen) {
            self.pos += 1;
            if self.peek() == Some(&Tok::RParen) {
                self.pos += 1;
            } else {
                loop {
                    args.push(self.sum()?);
                    match self.peek() {
                        Some(Tok::Comma) => self.pos += 1,
                        _ => {
                            self.expect(&Tok::RParen, "')' or ','")?;
                            break;
                        }
                    }
                }
            }
        } else {
            args.push(self.chain()?);
        }
        if args.len() != f.arity() {
            let n = f.arity();
            return Err(err(
                start..self.prev_end(),
                format!(
                    "{} takes {n} argument{}, got {}",
                    f.name(),
                    if n == 1 { "" } else { "s" },
                    args.len()
                ),
            ));
        }
        Ok(Expr::Call(f, args))
    }

    /// The argument of a function without parentheses: an implicit product
    /// that runs up to the next function name, operator or closing bracket.
    fn chain(&mut self) -> Res<Expr> {
        let base = self.depth;
        let mut lhs = self.power()?;
        while match self.peek() {
            Some(Tok::Ident(n)) => Func::from_name(n).is_none(),
            Some(Tok::LParen) => true,
            _ => false,
        } {
            let rhs = self.power()?;
            self.bump()?;
            lhs = Expr::Bin(BinOp::Mul, Box::new(lhs), Box::new(rhs));
        }
        self.depth = base;
        Ok(lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{free_params, print, random_expr};
    use crate::rng::Rng;

    fn p(src: &str) -> String {
        match parse_expr(src) {
            Ok(e) => print(&e),
            Err(e) => panic!("{src:?} failed: {e:?}"),
        }
    }

    fn fail(src: &str) -> ParseError {
        let e = parse(src).expect_err(src);
        assert!(!e.message.is_empty(), "{src:?}: empty message");
        e
    }

    #[test]
    fn precedence_and_implicit_multiplication() {
        let cases = [
            ("1+2*3", "1 + 2 * 3"),
            ("(1+2)*3", "(1 + 2) * 3"),
            ("1-2-3", "1 - 2 - 3"),
            ("1-(2-3)", "1 - (2 - 3)"),
            ("-x^2", "-(x ^ 2)"),
            ("2^3^2", "2 ^ (3 ^ 2)"),
            ("(2^3)^2", "(2 ^ 3) ^ 2"),
            ("2^-1", "2 ^ (-1)"),
            ("-2", "-2"),
            ("--x", "-(-x)"),
            ("2ab x", "2 * a * b * x"),
            ("k_r sin(wx)", "k_r * sin(w * x)"),
            ("1/2x", "1 / 2 * x"),
            ("sin 2x", "sin(2 * x)"),
            ("sin x cos x", "sin(x) * cos(x)"),
            ("sin x^2", "sin(x ^ 2)"),
            ("sin x / 2", "sin(x) / 2"),
            ("(x+1)(x-1)", "(x + 1) * (x - 1)"),
            ("3(x)", "3 * x"),
            ("a(x+1)", "a * (x + 1)"),
            ("cos t + 1", "cos(t) + 1"),
            ("theta", "θ"),
            ("2pi", "2 * π"),
            ("x·y", "x * y"),
            ("x×y", "x * y"),
            ("x−1", "x - 1"),
            ("max(a, b)", "max(a, b)"),
            ("e^x", "e ^ x"),
            ("sin(x)^2", "sin(x) ^ 2"),
            ("pix", "π * x"),
            ("sinx", "sin(x)"),
            ("exp x", "exp(x)"),
            ("1.5x + .5", "1.5 * x + 0.5"),
            ("a_1 b_2", "a_1 * b_2"),
            ("3ω t", "3 * ω * t"),
            ("theta_1", "θ_1"),
        ];
        for (src, want) in cases {
            assert_eq!(p(src), want, "{src}");
        }
    }

    #[test]
    fn never_folds_literals() {
        assert_eq!(
            parse_expr("-2").unwrap(),
            Expr::Neg(Box::new(Expr::Num(2.0)))
        );
    }

    #[test]
    fn definitions() {
        let body = |s| parse_expr(s).unwrap();
        assert_eq!(
            parse("y = sin(x)").unwrap(),
            Definition::Explicit {
                var: "x".into(),
                body: body("sin(x)")
            }
        );
        assert_eq!(
            parse("f(t) = t^2").unwrap(),
            Definition::Explicit {
                var: "t".into(),
                body: body("t^2")
            }
        );
        let polar = Definition::Polar {
            theta: "θ".into(),
            r: body("1 + cos(3θ)"),
        };
        assert_eq!(parse("r = 1 + cos(3θ)").unwrap(), polar);
        assert_eq!(parse("r = 1 + cos(3theta)").unwrap(), polar);
        assert_eq!(
            parse("(cos t, sin t)").unwrap(),
            Definition::Parametric {
                t: "t".into(),
                x: body("cos t"),
                y: body("sin t")
            }
        );
        assert_eq!(
            parse("sin(x)").unwrap(),
            Definition::Explicit {
                var: "x".into(),
                body: body("sin(x)")
            }
        );
        let e = fail("x = 3");
        assert!(e.message.starts_with("Write y ="), "{e:?}");
        fail("sin(x) = 3");
        fail("y = ");
        fail("(a, b, c)");
    }

    #[test]
    fn free_parameters() {
        let fp = |s: &str| free_params(&parse(s).unwrap());
        assert_eq!(fp("f(x) = a sin(b x)"), ["a", "b"]);
        assert_eq!(fp("r = k cos(θ) + pi"), ["k"]);
        assert_eq!(fp("(cos t, a sin t)"), ["a"]);
        assert_eq!(fp("y = e^x"), Vec::<String>::new());
    }

    #[test]
    fn error_spans() {
        let e = fail("sin(");
        assert_eq!(e.span, 4..4);
        // The second `^` (bytes 2..3) is where an operand was expected.
        assert_eq!(fail("2^^3").span, 2..3);
        assert_eq!(fail(")").span, 0..1);
        let e = fail("");
        assert_eq!((e.span, e.message.as_str()), (0..0, "type an expression"));
        assert_eq!(fail("ж").span, 0..2);
        assert_eq!(fail("sin()").span, 0..5);
        assert_eq!(fail("max(1)").span, 0..6);
        assert_eq!(fail("sin(1, 2)").span, 0..9);
        let e = fail("2 3");
        assert_eq!((e.span, e.message.as_str()), (2..3, "missing operator"));
        fail("(1 + 2");
        fail("x + ");
        fail("1.2.3");
    }

    #[test]
    fn deep_nesting_is_an_error_not_a_stack_overflow() {
        let src = "(".repeat(10_000) + "x" + &")".repeat(10_000);
        let e = parse(&src).unwrap_err();
        assert_eq!(e.message, "too deeply nested");
        let src = "-".repeat(10_000) + "x";
        assert!(parse(&src).is_err());
        let src = "sin ".repeat(10_000) + "x";
        assert!(parse(&src).is_err());
        let src = vec!["1"; 10_000].join("+");
        assert!(parse(&src).is_err());
        // Reasonable nesting still parses.
        let src = "(".repeat(200) + "x" + &")".repeat(200);
        assert!(parse(&src).is_ok());
    }

    #[test]
    fn print_parse_round_trip() {
        let mut rng = Rng::new(3);
        for _ in 0..500 {
            let e = random_expr(&mut rng, 5);
            let text = print(&e);
            assert_eq!(parse_expr(&text), Ok(e), "{text}");
        }
    }
}
