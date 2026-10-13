//! A small syntax highlighter for the exported code: keywords, strings,
//! comments, numbers, function calls, types and macros. Enough to read
//! comfortably; not a parser.

use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};

use crate::content::Lang;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Keyword,
    Str,
    Comment,
    Number,
    Function,
    Type,
    Macro,
}

const PY: &[&str] = &[
    "and", "as", "assert", "break", "class", "continue", "def", "del", "elif", "else", "except",
    "False", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "None",
    "nonlocal", "not", "or", "pass", "raise", "return", "True", "try", "while", "with", "yield",
];
const TS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "of",
    "readonly",
    "return",
    "static",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
    "yield",
    "number",
    "string",
    "boolean",
];
const RS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "else", "enum", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "f32", "f64", "i32", "i64", "u8", "u32", "u64", "usize", "bool",
];

/// Split source into colored tokens.
pub fn tokens(code: &str, lang: Lang) -> Vec<(Kind, &str)> {
    let keywords = match lang {
        Lang::Python => PY,
        Lang::TypeScript => TS,
        Lang::Rust => RS,
    };
    let b = code.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut plain_start = 0;
    while i < b.len() {
        let c = b[i];
        let rest = &code[i..];
        // Comments.
        let line_comment = match lang {
            Lang::Python => c == b'#',
            _ => rest.starts_with("//"),
        };
        let token: Option<(Kind, usize)> = if line_comment {
            Some((Kind::Comment, rest.find('\n').unwrap_or(rest.len())))
        } else if lang != Lang::Python && rest.starts_with("/*") {
            Some((
                Kind::Comment,
                rest[2..].find("*/").map_or(rest.len(), |e| e + 4),
            ))
        } else if lang == Lang::Python && (rest.starts_with("\"\"\"") || rest.starts_with("'''")) {
            let q = &rest[..3];
            Some((Kind::Str, rest[3..].find(q).map_or(rest.len(), |e| e + 6)))
        } else if c == b'"' || c == b'\'' || (c == b'`' && lang == Lang::TypeScript) {
            // Rust lifetimes ('a) are not strings.
            if c == b'\'' && lang == Lang::Rust && !is_char_literal(rest) {
                None
            } else {
                Some((Kind::Str, string_len(rest, c)))
            }
        } else if c.is_ascii_digit() && (i == 0 || !is_ident(b[i - 1])) {
            let n = rest
                .bytes()
                .take_while(|&d| d.is_ascii_alphanumeric() || d == b'.' || d == b'_')
                .count();
            // "0..10" is two numbers and a range, not one number.
            let n = rest[..n].find("..").unwrap_or(n);
            Some((Kind::Number, n))
        } else if is_ident_start(c) && (i == 0 || !is_ident(b[i - 1])) {
            let n = rest.bytes().take_while(|&d| is_ident(d)).count();
            let word = &rest[..n];
            let next = rest[n..].trim_start_matches(' ').bytes().next();
            let kind = if keywords.contains(&word) {
                Kind::Keyword
            } else if lang == Lang::Rust && rest[n..].starts_with('!') {
                Kind::Macro
            } else if next == Some(b'(') {
                Kind::Function
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                Kind::Type
            } else {
                Kind::Plain
            };
            Some((kind, if kind == Kind::Macro { n + 1 } else { n }))
        } else if (lang == Lang::Rust && rest.starts_with("#["))
            || (lang == Lang::Python && c == b'@' && (i == 0 || b[i - 1] == b'\n'))
        {
            Some((
                Kind::Macro,
                rest.find(['\n', '(']).unwrap_or(rest.len()).max(1),
            ))
        } else {
            None
        };
        match token {
            Some((kind, len)) if kind != Kind::Plain || len > 0 => {
                let len = len.max(1).min(rest.len());
                flush(&mut out, plain_start, i, code);
                out.push((kind, &code[i..i + len]));
                i += len;
                plain_start = i;
            }
            _ => {
                i += rest.chars().next().map_or(1, char::len_utf8);
            }
        }
    }
    flush(&mut out, plain_start, b.len(), code);
    out
}

fn flush<'a>(out: &mut Vec<(Kind, &'a str)>, from: usize, to: usize, code: &'a str) {
    if to > from {
        out.push((Kind::Plain, &code[from..to]));
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_char_literal(rest: &str) -> bool {
    let b = rest.as_bytes();
    (b.len() >= 3 && b[2] == b'\'') || (b.len() >= 4 && b[1] == b'\\' && b[3] == b'\'')
}

/// Length of a quoted string starting at `rest`, honoring backslash escapes.
fn string_len(rest: &str, quote: u8) -> usize {
    let b = rest.as_bytes();
    let mut i = 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            c if c == quote => return i + 1,
            b'\n' if quote != b'`' => return i,
            _ => i += 1,
        }
    }
    b.len()
}

pub fn color(kind: Kind, dark: bool, plain: Color32) -> Color32 {
    let (d, l) = match kind {
        Kind::Plain => return plain,
        Kind::Keyword => (0xC792EA, 0x7A3E9D),
        Kind::Str => (0xC3E88D, 0x3E7D1F),
        Kind::Comment => (0x6E7B89, 0x8A8578),
        Kind::Number => (0xF78C6C, 0xB5532A),
        Kind::Function => (0x82AAFF, 0x2F5DB8),
        Kind::Type => (0xFFCB6B, 0x8F5E00),
        Kind::Macro => (0x89DDFF, 0x0E7490),
    };
    let v: u32 = if dark { d } else { l };
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// The highlighted code as a layout job in the monospace font.
pub fn layout(code: &str, lang: Lang, dark: bool, plain: Color32, size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (kind, text) in tokens(code, lang) {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: FontId::monospace(size),
                color: color(kind, dark, plain),
                italics: kind == Kind::Comment,
                ..Default::default()
            },
        );
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str, lang: Lang) -> Vec<(Kind, String)> {
        tokens(code, lang)
            .into_iter()
            .filter(|(k, t)| *k != Kind::Plain || !t.trim().is_empty())
            .map(|(k, t)| (k, t.trim().to_owned()))
            .collect()
    }

    #[test]
    fn tokens_cover_the_source_exactly() {
        for (code, lang) in [
            ("def f(x):\n    return x * 2  # double\n", Lang::Python),
            (
                "const s: string = `a${b}`; // hi\nlet n = 0x1F;",
                Lang::TypeScript,
            ),
            (
                "fn main<'a>(x: &'a str) { println!(\"{}\", 'c'); /* c */ }",
                Lang::Rust,
            ),
        ] {
            let joined: String = tokens(code, lang).iter().map(|(_, t)| *t).collect();
            assert_eq!(joined, code);
        }
    }

    #[test]
    fn python() {
        let k = kinds(
            "import numpy as np\nx = lerp(1.5, 2)  # mix\ns = \"a#b\"",
            Lang::Python,
        );
        assert!(k.contains(&(Kind::Keyword, "import".into())));
        assert!(k.contains(&(Kind::Function, "lerp".into())));
        assert!(k.contains(&(Kind::Number, "1.5".into())));
        assert!(k.contains(&(Kind::Comment, "# mix".into())));
        assert!(k.contains(&(Kind::Str, "\"a#b\"".into())));
    }

    #[test]
    fn rust() {
        let k = kinds(
            "#[macroquad::main(\"T\")]\nlet v: Vec2 = vec2(0.0, 1.0); draw_text!(x); for i in 0..10 {}",
            Lang::Rust,
        );
        assert!(k.contains(&(Kind::Macro, "#[macroquad::main".into())));
        assert!(k.contains(&(Kind::Type, "Vec2".into())));
        assert!(k.contains(&(Kind::Function, "vec2".into())));
        assert!(k.contains(&(Kind::Macro, "draw_text!".into())));
        assert!(k.contains(&(Kind::Number, "0".into())));
        assert!(k.contains(&(Kind::Number, "10".into())));
    }
}
