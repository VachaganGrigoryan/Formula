//! Code export for the plotter: the content files are templates whose
//! `@PARAMS` and `@EXPRESSIONS` regions are regenerated from the live rows.
//!
//! The generated code names the parameter table `vals`, which can never be a
//! parameter: names are single letters, subscripted letters or Greek words.

use super::{Expression, Param};
use crate::content::Lang;
use crate::expr::{self, Definition, Expr};

/// Replaces each marker region of `template` (a `# @NAME` or `// @NAME`
/// line, up to the next `@END` line, markers included) with its text.
pub fn fill(template: &str, regions: &[(&str, String)]) -> String {
    let marker = |line: &str| {
        let t = line.trim();
        t.strip_prefix("#")
            .or_else(|| t.strip_prefix("//"))
            .map(|rest| rest.trim().to_string())
            .filter(|rest| rest.starts_with('@'))
    };
    let mut out = String::new();
    let mut skipping = false;
    for line in template.lines() {
        match marker(line) {
            Some(m) if skipping && m == "@END" => skipping = false,
            Some(m) if !skipping => {
                if let Some((_, text)) = regions.iter().find(|(name, _)| m == format!("@{name}")) {
                    for l in text.lines() {
                        out.push_str(l);
                        out.push('\n');
                    }
                    skipping = true;
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            _ if skipping => {}
            _ => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out
}

/// The parameter table entries, one line per slider.
pub fn params(e: &Expression, lang: Lang) -> String {
    e.params
        .iter()
        .map(|(name, p)| {
            let id = expr::ident(name, lang);
            let Param { value, min, .. } = *p;
            // A slider needs a non-empty range.
            let max = if p.max > min { p.max } else { min + 1.0 };
            match lang {
                Lang::Python => format!("    \"{id}\": ({value:?}, {min:?}, {max:?}),"),
                Lang::TypeScript => {
                    format!("  {id}: {{ value: {value:?}, min: {min:?}, max: {max:?} }},")
                }
                Lang::Rust => format!(
                    "        Param {{ name: \"{id}\", value: {value:?}, min: {min:?}, max: {max:?} }},"
                ),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The body of the function that builds every row, in `lang`.
pub fn expressions(e: &Expression, lang: Lang) -> String {
    let names: Vec<String> = e.params.keys().map(|n| expr::ident(n, lang)).collect();
    let rows: Vec<(usize, &Definition, (f64, f64))> = e
        .rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.def.as_ref().ok().map(|d| (i + 1, d, r.t_range)))
        .collect();
    let code = |e: &Expr| expr::to_code(e, lang);
    let var = |v: &str| expr::ident(v, lang);
    match lang {
        Lang::Python => {
            let mut blocks = Vec::new();
            if !names.is_empty() {
                let lookups: Vec<String> = names.iter().map(|n| format!("vals[\"{n}\"]")).collect();
                blocks.push(format!("    {} = {}", names.join(", "), lookups.join(", ")));
            }
            let mut calls = Vec::new();
            for (n, def, (t0, t1)) in &rows {
                let def_fn = |name: String, v: &str, body: &Expr| {
                    format!("    def {name}({}):\n        return {}", var(v), code(body))
                };
                match def {
                    Definition::Explicit { var: v, body } => {
                        blocks.push(def_fn(format!("row{n}"), v, body));
                        calls.push(format!("explicit(row{n}, xs)"));
                    }
                    Definition::Polar { theta, r } => {
                        blocks.push(def_fn(format!("row{n}"), theta, r));
                        calls.push(format!("polar(row{n}, {t0:?}, {t1:?})"));
                    }
                    Definition::Parametric { t, x, y } => {
                        blocks.push(def_fn(format!("row{n}_x"), t, x));
                        blocks.push(def_fn(format!("row{n}_y"), t, y));
                        calls.push(format!("parametric(row{n}_x, row{n}_y, {t0:?}, {t1:?})"));
                    }
                }
            }
            blocks.push(format!("    return [{}]", calls.join(", ")));
            blocks.join("\n\n")
        }
        Lang::TypeScript => {
            let mut lines = Vec::new();
            if !names.is_empty() {
                lines.push(format!("  const {{ {} }} = vals;", names.join(", ")));
            }
            lines.push("  return [".into());
            for (_, def, (t0, t1)) in &rows {
                lines.push(match def {
                    Definition::Explicit { var: v, body } => format!(
                        "    {{ kind: \"explicit\", f: ({}: number) => {} }},",
                        var(v),
                        code(body)
                    ),
                    Definition::Polar { theta, r } => {
                        let th = var(theta);
                        format!(
                            "    {{ kind: \"curve\", f: ({th}: number) => polar({}, {th}), t0: {t0:?}, t1: {t1:?} }},",
                            code(r)
                        )
                    }
                    Definition::Parametric { t, x, y } => format!(
                        "    {{ kind: \"curve\", f: ({}: number): [number, number] => [{}, {}], t0: {t0:?}, t1: {t1:?} }},",
                        var(t),
                        code(x),
                        code(y)
                    ),
                });
            }
            lines.push("  ];".into());
            lines.join("\n")
        }
        Lang::Rust => {
            let mut lines: Vec<String> = names
                .iter()
                .map(|n| format!("    let {n} = value(vals, \"{n}\");"))
                .collect();
            if names.is_empty() {
                lines.push("    let _ = vals;".into());
            }
            lines.push("    vec![".into());
            // A closure argument the body never reads is `_x`, so the export compiles without warnings.
            let arg = |v: &str, used: bool| {
                if used { var(v) } else { format!("_{}", var(v)) }
            };
            for (_, def, (t0, t1)) in &rows {
                lines.push(match def {
                    Definition::Explicit { var: v, body } => format!(
                        "        Row::Explicit(Box::new(move |{}: f64| {})),",
                        arg(v, uses(body, v)),
                        code(body)
                    ),
                    Definition::Polar { theta, r } => {
                        let th = var(theta);
                        format!(
                            "        Row::Curve(Box::new(move |{th}: f64| polar({}, {th})), {t0:?}, {t1:?}),",
                            code(r)
                        )
                    }
                    Definition::Parametric { t, x, y } => format!(
                        "        Row::Curve(Box::new(move |{}: f64| ({}, {})), {t0:?}, {t1:?}),",
                        arg(t, uses(x, t) || uses(y, t)),
                        code(x),
                        code(y)
                    ),
                });
            }
            lines.push("    ]".into());
            lines.join("\n")
        }
    }
}

/// Whether `e` mentions the variable `name`.
fn uses(e: &Expr, name: &str) -> bool {
    match e {
        Expr::Num(_) => false,
        Expr::Var(n) => n == name,
        Expr::Neg(a) => uses(a, name),
        Expr::Bin(_, a, b) => uses(a, name) || uses(b, name),
        Expr::Call(_, args) => args.iter().any(|a| uses(a, name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::Formula;

    /// Rows that use every function, Greek names, subscripts, constants and
    /// all three kinds of definition.
    pub(crate) const RICH: [&str; 6] = [
        "y = a sin(bx) + cos x tan x + asin(x/9) acos(x/9) atan x",
        "f(t) = sqrt(abs t) + exp(-t^2) + ln(1 + t^2) log(2 + t)",
        "y = floor(x) + ceil(x)/2 + min(x, k_r) + max(-x, e)",
        "r = 1 + cos(λθ) / 2",
        "(cos t, ω sin(2t))",
        "y = pi",
    ];

    fn rich() -> Expression {
        let mut e = Expression::with_rows(&RICH);
        e.params.get_mut("a").unwrap().value = 2.0;
        e
    }

    #[test]
    fn fill_replaces_marked_regions_only() {
        let t = "head\n# @A\nold\n# @END\nmid\n// @B\nold\n// @END\n// @C stays\ntail\n";
        let out = fill(t, &[("A", "new a".into()), ("B", "b1\nb2".into())]);
        assert_eq!(out, "head\nnew a\nmid\nb1\nb2\n// @C stays\ntail\n");
    }

    #[test]
    fn python_export_has_the_values_and_expressions() {
        let mut e = Expression::with_rows(&["y = a*sin(b*x)"]);
        e.params.get_mut("a").unwrap().value = 2.0;
        let py = e.export_code(Lang::Python).unwrap();
        assert!(py.contains("\"a\": (2.0, -10.0, 10.0),"), "{py}");
        assert!(py.contains("return a * np.sin(b * x)"), "{py}");
        assert!(py.contains("a, b = vals[\"a\"], vals[\"b\"]"), "{py}");
        assert!(!py.contains("@PARAMS") && !py.contains("@EXPRESSIONS") && !py.contains("@END"));
    }

    #[test]
    fn every_language_is_ascii_for_rich_rows() {
        let e = rich();
        assert!(e.rows.iter().all(|r| r.def.is_ok()), "all rich rows parse");
        for lang in Lang::ALL {
            let code = e.export_code(lang).unwrap();
            if let Some((n, line)) = code.lines().enumerate().find(|(_, l)| !l.is_ascii()) {
                panic!("{lang:?} line {}: {line}", n + 1);
            }
            assert!(code.contains("lam"), "{lang:?}: λ becomes lam");
            assert!(code.contains("omega") && code.contains("k_r"), "{lang:?}");
        }
    }

    #[test]
    fn the_static_files_are_the_default_export() {
        let e = Expression::default();
        for lang in Lang::ALL {
            let template = crate::content::code("expression", lang).unwrap();
            let unmarked: String = template
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !(t.starts_with("# @") || t.starts_with("// @"))
                })
                .map(|l| format!("{l}\n"))
                .collect();
            assert_eq!(e.export_code(lang).unwrap(), unmarked, "{lang:?}");
        }
    }

    #[test]
    fn broken_and_blank_rows_are_left_out() {
        let e = Expression::with_rows(&["y = x", "sin(", ""]);
        let alone = Expression::with_rows(&["y = x"]);
        for lang in Lang::ALL {
            assert_eq!(e.export_code(lang), alone.export_code(lang), "{lang:?}");
        }
        let none = Expression::with_rows(&[""]);
        assert!(
            none.export_code(Lang::Python)
                .unwrap()
                .contains("    return []")
        );
        assert!(
            none.export_code(Lang::Rust)
                .unwrap()
                .contains("let _ = vals;")
        );
    }

    #[test]
    fn unused_rust_closure_arguments_are_underscored() {
        let e = Expression::with_rows(&["y = 2", "(1, 2)"]);
        let rs = e.export_code(Lang::Rust).unwrap();
        assert!(rs.contains("move |_x: f64| 2.0"), "{rs}");
        assert!(rs.contains("move |_t: f64| (1.0, 2.0)"), "{rs}");
    }

    /// Writes the rich exports for checking with the real toolchains:
    /// `EXPR_EXPORT_DIR=/tmp/x cargo test write_sample_exports -- --ignored`.
    #[test]
    #[ignore]
    fn write_sample_exports() {
        let dir = std::env::var("EXPR_EXPORT_DIR").expect("set EXPR_EXPORT_DIR");
        let e = rich();
        for lang in Lang::ALL {
            let path = std::path::Path::new(&dir).join(lang.file_name());
            std::fs::write(path, e.export_code(lang).unwrap()).unwrap();
        }
    }
}
