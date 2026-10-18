//! Typesetting the app's equations as LaTeX, for papers, homework and notebooks.

use crate::formula::{Formula, MathText, Piece};

/// The equation as a LaTeX math-mode string (without `$` delimiters).
pub fn latex_of(eq: &MathText) -> String {
    let chars: Vec<(Kind, char)> = eq
        .pieces()
        .flat_map(|piece| {
            let (kind, s) = match piece {
                Piece::Text(s) => (Kind::Text, s),
                Piece::Sub(s) => (Kind::Sub, s),
                Piece::Sup(s) => (Kind::Sup, s),
            };
            s.chars().map(move |c| (kind, c))
        })
        .collect();
    convert(&chars)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Sub,
    Sup,
}

/// The formula's equations with its current parameter values, as a block
/// ready to paste into a LaTeX document.
pub fn latex_snippet(f: &dyn Formula) -> String {
    snippet(&f.equations(), &f.params())
}

/// Equations, one per line, then a `where` line with the parameter values.
pub fn snippet<S: AsRef<str>>(equations: &[MathText], params: &[(S, f64)]) -> String {
    let mut lines: Vec<String> = equations.iter().map(latex_of).collect();
    if !params.is_empty() {
        let values: Vec<String> = params
            .iter()
            .map(|(name, v)| format!("{} = {}", param_name(name.as_ref()), number(*v)))
            .collect();
        lines.push(format!(r"\text{{where }} {}", values.join(r",\ ")));
    }
    format!(
        "\\begin{{gathered}}\n{}\n\\end{{gathered}}",
        lines.join(" \\\\\n")
    )
}

/// `"k_r"` → `k_{r}`: the part after an underscore is a subscript.
fn param_name(name: &str) -> String {
    let eq = match name.split_once('_') {
        Some((base, sub)) => MathText::new().t(base).sub(sub),
        None => MathText::new().t(name),
    };
    latex_of(&eq)
}

/// A parameter value with at most four significant digits.
pub fn number(x: f64) -> String {
    if x == 0.0 || !x.is_finite() {
        return if x == 0.0 { "0".into() } else { format!("{x}") };
    }
    if x.abs() < 1e-4 {
        let s = format!("{x:.3e}");
        let (mantissa, exponent) = s.split_once('e').unwrap_or((&s, "0"));
        return format!(r"{} \times 10^{{{exponent}}}", trim_zeros(mantissa));
    }
    let digits = x.abs().log10().floor() as i32 + 1;
    let decimals = (4 - digits).max(0) as usize;
    trim_zeros(&format!("{x:.decimals$}")).to_owned()
}

/// `"2.500"` → `"2.5"`, `"3.000"` → `"3"`.
fn trim_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// Words typeset upright as LaTeX operators (`\sin`), not as italic letters.
const FUNCTIONS: [&str; 18] = [
    "sin", "cos", "tan", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh", "log", "ln", "exp",
    "min", "max", "det", "lim", "gcd", "arg",
];

/// Two-letter English words. Other two-letter runs (`dx`, `xy`) are products
/// of variables; anything longer that is not a function is prose.
const SHORT_WORDS: [&str; 14] = [
    "is", "in", "on", "of", "or", "at", "to", "an", "as", "be", "by", "if", "it", "no",
];

/// The LaTeX for a character that is not written as itself.
fn symbol(c: char) -> Option<&'static str> {
    Some(match c {
        'α' => r"\alpha",
        'β' => r"\beta",
        'γ' => r"\gamma",
        'δ' => r"\delta",
        'Δ' => r"\Delta",
        'ε' => r"\varepsilon",
        'θ' => r"\theta",
        'κ' => r"\kappa",
        'λ' => r"\lambda",
        'μ' => r"\mu",
        'π' => r"\pi",
        'ρ' => r"\rho",
        'σ' => r"\sigma",
        'Σ' => r"\sum",
        'τ' => r"\tau",
        'φ' => r"\varphi",
        'ω' => r"\omega",
        '·' => r"\cdot",
        '×' => r"\times",
        '−' => "-",
        '±' => r"\pm",
        '≤' => r"\le",
        '≥' => r"\ge",
        '≠' => r"\ne",
        '≈' => r"\approx",
        '∞' => r"\infty",
        '…' => r"\ldots",
        '²' => "^{2}",
        '³' => "^{3}",
        '%' => r"\%",
        '&' => r"\&",
        '#' => r"\#",
        '$' => r"\$",
        '_' => r"\_",
        '{' => r"\{",
        '}' => r"\}",
        _ => return None,
    })
}

fn convert(chars: &[(Kind, char)]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let (kind, c) = chars[i];
        if kind != Kind::Text {
            let end = run_end(chars, i, |(k, _)| k == kind);
            let inner: Vec<_> = chars[i..end]
                .iter()
                .map(|&(_, c)| (Kind::Text, c))
                .collect();
            out.push_str(if kind == Kind::Sub { "_{" } else { "^{" });
            out.push_str(&convert(&inner));
            out.push('}');
            i = end;
            continue;
        }
        if c == ' ' {
            let end = run_end(chars, i, |p| p == (Kind::Text, ' '));
            if end - i >= 2 {
                push_command(&mut out, r"\quad", chars.get(end));
                i = end;
            } else if prose_at(chars, end).is_some() {
                i = text_run(chars, i, &mut out);
            } else {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c.is_ascii_alphabetic() {
            if prose_at(chars, i).is_some() {
                i = text_run(chars, i, &mut out);
                continue;
            }
            let end = word_end(chars, i);
            let word: String = chars[i..end].iter().map(|&(_, c)| c).collect();
            if FUNCTIONS.contains(&word.as_str()) {
                push_command(&mut out, &format!("\\{word}"), chars.get(end));
            } else {
                out.push_str(&word);
            }
            i = end;
            continue;
        }
        if c == '√' {
            let open = run_end(chars, i + 1, |(_, c)| c == ' ');
            if let Some(close) = matching_paren(chars, open) {
                out.push_str(r"\sqrt{");
                out.push_str(&convert(&chars[open + 1..close]));
                out.push('}');
                i = close + 1;
            } else {
                push_command(&mut out, r"\sqrt", chars.get(i + 1));
                i += 1;
            }
            continue;
        }
        match symbol(c) {
            Some(tex) => push_command(&mut out, tex, chars.get(i + 1)),
            None => out.push(c),
        }
        i += 1;
    }
    out
}

/// The end of the run of plain-text letters starting at `i`.
fn word_end(chars: &[(Kind, char)], i: usize) -> usize {
    run_end(chars, i, |(k, c)| {
        k == Kind::Text && c.is_ascii_alphabetic()
    })
}

/// The end of the run of characters from `i` that satisfy `f`.
fn run_end(chars: &[(Kind, char)], i: usize, f: impl Fn((Kind, char)) -> bool) -> usize {
    i + chars[i..].iter().take_while(|&&p| f(p)).count()
}

/// If an English word starts at `i`, where it ends.
fn prose_at(chars: &[(Kind, char)], i: usize) -> Option<usize> {
    let starts_word = chars
        .get(i)
        .is_some_and(|&(k, c)| k == Kind::Text && c.is_ascii_alphabetic());
    let mid_word = i > 0 && chars[i - 1].1.is_ascii_alphanumeric();
    if !starts_word || mid_word {
        return None;
    }
    let end = word_end(chars, i);
    let word: String = chars[i..end].iter().map(|&(_, c)| c).collect();
    let prose = (word.len() >= 3 && !FUNCTIONS.contains(&word.as_str()))
        || SHORT_WORDS.contains(&word.as_str());
    prose.then_some(end)
}

/// Write the words (and the single spaces between and around them) starting
/// at `i` as one `\text{…}`, so they keep their spacing. Returns where it ends.
fn text_run(chars: &[(Kind, char)], mut i: usize, out: &mut String) -> usize {
    let mut text = String::new();
    loop {
        if let Some(end) = prose_at(chars, i) {
            text.extend(chars[i..end].iter().map(|&(_, c)| c));
            i = end;
        } else if chars.get(i) == Some(&(Kind::Text, ' ')) {
            let next = chars.get(i + 1).map(|&(_, c)| c);
            if prose_at(chars, i + 1).is_some() {
                text.push(' ');
                i += 1;
            } else if next.is_some_and(|c| c != ' ') {
                text.push(' ');
                i += 1;
                break;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    out.push_str(&format!("\\text{{{text}}}"));
    i
}

/// Append a command, with a space if the next character would otherwise run
/// into its name (`\theta x`, not `\thetax`).
fn push_command(out: &mut String, tex: &str, next: Option<&(Kind, char)>) {
    out.push_str(tex);
    let ends_in_letter = tex.ends_with(|c: char| c.is_ascii_alphabetic());
    if ends_in_letter && next.is_some_and(|&(k, c)| k == Kind::Text && c.is_ascii_alphanumeric()) {
        out.push(' ');
    }
}

/// The index of the `)` closing the `(` at `open`, if `open` is one.
fn matching_paren(chars: &[(Kind, char)], open: usize) -> Option<usize> {
    if chars.get(open).map(|&(_, c)| c) != Some('(') {
        return None;
    }
    let mut depth = 0;
    for (k, &(_, c)) in chars.iter().enumerate().skip(open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(k);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::{TOOLS, ToolEnv};

    fn tex(eq: MathText) -> String {
        latex_of(&eq)
    }

    #[test]
    fn functions_and_greek_letters_become_commands() {
        assert_eq!(tex(MathText::new().t("x = r cos θ")), r"x = r \cos \theta");
    }

    #[test]
    fn subscripts_and_superscripts_are_braced() {
        assert_eq!(
            tex(MathText::new().t("v").sub("0").t(" + t²")),
            "v_{0} + t^{2}"
        );
        assert_eq!(tex(MathText::new().t("e").sup("iθ")), r"e^{i\theta}");
    }

    #[test]
    fn square_root_takes_its_parenthesised_argument() {
        assert_eq!(
            tex(MathText::new().t("d = √((x2 − x1)² + 1)")),
            r"d = \sqrt{(x2 - x1)^{2} + 1}"
        );
    }

    #[test]
    fn square_root_argument_can_span_subscripts() {
        assert_eq!(
            tex(MathText::new().t("√( x").sub("2").t(" )")),
            r"\sqrt{ x_{2} }"
        );
    }

    #[test]
    fn prose_is_set_as_upright_text_with_its_spaces() {
        assert_eq!(
            tex(MathText::new()
                .t("c is in the set while |z")
                .sub("n")
                .t("| ≤ 2")),
            r"c\text{ is in the set while }|z_{n}| \le 2"
        );
        assert_eq!(tex(MathText::new().t("X").sub("norm")), r"X_{\text{norm}}");
    }

    #[test]
    fn wide_gaps_become_quads() {
        assert_eq!(
            tex(MathText::new().t("Time   O(n²)")),
            r"\text{Time}\quad O(n^{2})"
        );
    }

    #[test]
    fn short_math_words_stay_math() {
        assert_eq!(tex(MathText::new().t("dz/dt = xy")), "dz/dt = xy");
    }

    #[test]
    fn no_space_before_a_subscript() {
        assert_eq!(tex(MathText::new().t("μ").sub("x")), r"\mu_{x}");
    }

    #[test]
    fn latex_special_characters_are_escaped() {
        assert_eq!(tex(MathText::new().t("50% & #1")), r"50\% \& \#1");
    }

    #[test]
    fn numbers_keep_four_significant_digits() {
        assert_eq!(number(3.0), "3");
        assert_eq!(number(28.0), "28");
        assert_eq!(number(-0.5), "-0.5");
        assert_eq!(number(8.0 / 3.0), "2.667");
        assert_eq!(number(0.123456), "0.1235");
        assert_eq!(number(12345.6), "12346");
        assert_eq!(number(0.0000012), r"1.2 \times 10^{-6}");
    }

    #[test]
    fn snippet_lists_the_parameter_values() {
        let eqs = [MathText::new().t("y = a x"), MathText::new().t("z = θ")];
        assert_eq!(
            snippet(&eqs, &[("a", 2.0), ("k_r", 0.5), ("θ", 1.0)]),
            "\\begin{gathered}\ny = a x \\\\\nz = \\theta \\\\\n\
             \\text{where } a = 2,\\ k_{r} = 0.5,\\ \\theta = 1\n\\end{gathered}"
        );
    }

    #[test]
    fn snippet_without_parameters_has_no_where_line() {
        assert_eq!(
            snippet::<&str>(&[MathText::new().t("y = x")], &[]),
            "\\begin{gathered}\ny = x\n\\end{gathered}"
        );
    }

    /// The tools driven by numeric sliders report them, so the LaTeX shows
    /// the values the viewport is using.
    #[test]
    fn slider_tools_report_their_parameters() {
        let env = ToolEnv::new();
        for id in [
            "polar-to-cartesian",
            "perspective-projection",
            "lerp",
            "cubic-bezier",
            "lissajous",
            "force-layout",
            "min-max-normalization",
            "flow-field",
            "lorenz",
            "rotation-matrix",
            "ease-out-cubic",
            "relative-luminance",
        ] {
            let tool = crate::formulas::tool(id).unwrap();
            let f = (tool.make)(&env);
            let params = f.params();
            assert!(!params.is_empty(), "{id} reports no parameters");
            let tex = latex_snippet(&*f);
            for (name, _) in params {
                let name = snippet(&[], &[(name.as_str(), 0.0)]);
                let name = name
                    .split("where } ")
                    .nth(1)
                    .unwrap()
                    .split(" =")
                    .next()
                    .unwrap();
                assert!(tex.contains(name), "{id}: {name} missing from {tex}");
            }
        }
    }

    #[test]
    fn every_snippet_is_ascii_and_balanced() {
        let env = ToolEnv::new();
        for tool in TOOLS {
            let tex = latex_snippet(&*(tool.make)(&env));
            assert!(tex.is_ascii(), "{}: {tex}", tool.id);
            assert!(balanced(&tex), "{}: {tex}", tool.id);
        }
    }

    fn balanced(s: &str) -> bool {
        let mut depth = 0i32;
        let mut escaped = false;
        for c in s.chars() {
            match c {
                '\\' => escaped = !escaped,
                '{' if !escaped => depth += 1,
                '}' if !escaped => depth -= 1,
                _ => escaped = false,
            }
            if c != '\\' {
                escaped = false;
            }
            if depth < 0 {
                return false;
            }
        }
        depth == 0
    }

    /// Every equation in the app converts to plain ASCII LaTeX, so a symbol a
    /// new formula introduces cannot slip through unmapped.
    #[test]
    fn every_equation_converts_to_ascii_latex() {
        let env = ToolEnv::new();
        for tool in TOOLS {
            for eq in (tool.make)(&env).equations() {
                let out = latex_of(&eq);
                assert!(!out.is_empty(), "{}: {:?} is empty", tool.id, eq.text());
                let bad: String = out.chars().filter(|c| !c.is_ascii()).collect();
                assert!(bad.is_empty(), "{}: unmapped {bad:?} in {out}", tool.id);
                assert!(balanced(&out), "{}: unbalanced braces in {out}", tool.id);
            }
        }
    }
}
