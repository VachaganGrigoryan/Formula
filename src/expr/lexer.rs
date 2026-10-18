//! Turns source text into tokens, splitting letter runs into variables.

use super::ast::{Func, ParseError};
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Tok {
    Num(f64),
    /// A canonical variable or function name.
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Eq,
}

#[derive(Clone, Debug)]
pub(super) struct Token {
    pub kind: Tok,
    pub span: Range<usize>,
}

/// Greek names that stay whole and become their glyph.
const GREEK: [(&str, &str); 13] = [
    ("theta", "θ"),
    ("alpha", "α"),
    ("beta", "β"),
    ("gamma", "γ"),
    ("delta", "δ"),
    ("lambda", "λ"),
    ("mu", "μ"),
    ("omega", "ω"),
    ("phi", "φ"),
    ("psi", "ψ"),
    ("rho", "ρ"),
    ("sigma", "σ"),
    ("tau", "τ"),
];

/// The canonical name of a constant or Greek word, if `word` is one.
fn symbol_word(word: &str) -> Option<&'static str> {
    if word == "pi" {
        return Some("π");
    }
    GREEK.iter().find(|(w, _)| *w == word).map(|(_, g)| *g)
}

/// The length of the longest known word (function, Greek name, `pi`) that
/// `s` starts with.
fn longest_word(s: &str) -> Option<usize> {
    let funcs = Func::ALL.iter().map(|f| f.name());
    let greek = GREEK.iter().map(|(w, _)| *w);
    funcs
        .chain(greek)
        .chain(["pi"])
        .filter(|w| s.starts_with(w))
        .map(str::len)
        .max()
}

fn is_greek(c: char) -> bool {
    ('\u{391}'..='\u{3A9}').contains(&c) || ('\u{3B1}'..='\u{3C9}').contains(&c)
}

fn err(span: Range<usize>, message: impl Into<String>) -> ParseError {
    ParseError {
        span,
        message: message.into(),
    }
}

pub(super) fn lex(src: &str) -> Result<Vec<Token>, ParseError> {
    let mut toks = Vec::new();
    let mut it = src.char_indices().peekable();
    while let Some(&(i, c)) = it.peek() {
        let len = c.len_utf8();
        let simple = match c {
            '+' => Some(Tok::Plus),
            '-' | '\u{2212}' => Some(Tok::Minus),
            '*' | '\u{B7}' | '\u{D7}' => Some(Tok::Star),
            '/' | '\u{F7}' => Some(Tok::Slash),
            '^' => Some(Tok::Caret),
            '(' => Some(Tok::LParen),
            ')' => Some(Tok::RParen),
            ',' => Some(Tok::Comma),
            '=' => Some(Tok::Eq),
            _ => None,
        };
        if let Some(kind) = simple {
            it.next();
            toks.push(Token {
                kind,
                span: i..i + len,
            });
        } else if c.is_whitespace() {
            it.next();
        } else if c.is_ascii_digit()
            || (c == '.' && src[i + 1..].starts_with(|d: char| d.is_ascii_digit()))
        {
            let mut end = i;
            let mut dot = false;
            while let Some(&(j, d)) = it.peek() {
                let fraction =
                    d == '.' && !dot && src[j + 1..].starts_with(|n: char| n.is_ascii_digit());
                if d.is_ascii_digit() || fraction {
                    dot |= d == '.';
                    end = j + 1;
                    it.next();
                } else {
                    break;
                }
            }
            let v: f64 = src[i..end]
                .parse()
                .map_err(|_| err(i..end, "invalid number"))?;
            if !v.is_finite() {
                return Err(err(i..end, "number too large"));
            }
            toks.push(Token {
                kind: Tok::Num(v),
                span: i..end,
            });
        } else if c.is_ascii_alphabetic() || is_greek(c) {
            let mut end = i;
            while let Some(&(j, d)) = it.peek() {
                if d.is_ascii_alphabetic() || (j == i && is_greek(d)) {
                    end = j + d.len_utf8();
                    it.next();
                } else {
                    break;
                }
            }
            // A subscript (`_r`, `_1`) binds to the last letter of the run.
            let mut sub = None;
            if src[end..].starts_with('_') {
                let digits = src[end + 1..]
                    .find(|d: char| !d.is_ascii_alphanumeric())
                    .unwrap_or(src.len() - end - 1);
                if digits > 0 {
                    sub = Some(end + 1..end + 1 + digits);
                    while it.peek().is_some_and(|&(j, _)| j < end + 1 + digits) {
                        it.next();
                    }
                }
            }
            split_run(src, i..end, sub, &mut toks);
        } else {
            return Err(err(i..i + len, format!("unexpected character '{c}'")));
        }
    }
    Ok(toks)
}

/// Splits a letter run into tokens: known words stay whole (longest match
/// first), every other letter is its own variable.
fn split_run(src: &str, run: Range<usize>, sub: Option<Range<usize>>, toks: &mut Vec<Token>) {
    let mut push = |name: String, span: Range<usize>| {
        toks.push(Token {
            kind: Tok::Ident(name),
            span,
        });
    };
    let end_all = sub.as_ref().map_or(run.end, |s| s.end);
    let text = &src[run.clone()];
    // Whole-run symbol word with a subscript: `theta_1`, `pi_2`.
    if let (Some(s), Some(g)) = (&sub, symbol_word(text)) {
        push(format!("{g}_{}", &src[s.clone()]), run.start..end_all);
        return;
    }
    // With a subscript the last character is held back to carry it.
    let last_start = if sub.is_some() {
        run.end - text.chars().next_back().map_or(0, char::len_utf8)
    } else {
        run.end
    };
    let mut i = run.start;
    while i < last_start {
        let rest = &src[i..last_start];
        let n = match longest_word(rest) {
            Some(n) => n,
            None => rest.chars().next().map_or(1, char::len_utf8),
        };
        let word = &src[i..i + n];
        let name = symbol_word(word).unwrap_or(word);
        push(name.to_string(), i..i + n);
        i += n;
    }
    if let Some(s) = sub {
        let letter = &src[last_start..run.end];
        push(format!("{letter}_{}", &src[s.clone()]), last_start..s.end);
    }
}
