//! Reference documentation and exportable visualization code for every tool.
//!
//! The files live in `assets/content/<tool id>/` and are embedded at build
//! time (see `build.rs`).

/// One tool's embedded files.
pub struct Entry {
    pub id: &'static str,
    pub doc: Option<&'static str>,
    pub python: Option<&'static str>,
    pub typescript: Option<&'static str>,
    pub rust: Option<&'static str>,
}

include!(concat!(env!("OUT_DIR"), "/content.rs"));

/// A language the code export is available in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Python,
    TypeScript,
    Rust,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Python, Lang::TypeScript, Lang::Rust];

    pub fn label(self) -> &'static str {
        match self {
            Lang::Python => "Python",
            Lang::TypeScript => "TypeScript",
            Lang::Rust => "Rust",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Lang::Python => "main.py",
            Lang::TypeScript => "main.ts",
            Lang::Rust => "main.rs",
        }
    }

    /// How to run the exported file.
    pub fn run_hint(self) -> &'static str {
        match self {
            Lang::Python => "pip install numpy matplotlib  ·  python main.py",
            Lang::TypeScript => "tsc main.ts  ·  load main.js in any HTML page",
            Lang::Rust => "cargo new demo  ·  add macroquad = \"0.4\"  ·  cargo run --release",
        }
    }
}

fn entry(id: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.id == id)
}

/// The full reference documentation (markdown) for a tool.
pub fn doc(id: &str) -> Option<&'static str> {
    entry(id)?.doc
}

/// The exportable visualization code for a tool in one language.
pub fn code(id: &str, lang: Lang) -> Option<&'static str> {
    let e = entry(id)?;
    match lang {
        Lang::Python => e.python,
        Lang::TypeScript => e.typescript,
        Lang::Rust => e.rust,
    }
}

/// The section headings every doc has, in order.
pub const SECTIONS: [&str; 10] = [
    "Overview",
    "Formula",
    "Intuition",
    "Algorithm",
    "Complexity",
    "Parameters in the app",
    "Numerical notes and pitfalls",
    "Applications",
    "History",
    "References",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::TOOLS;

    #[test]
    fn every_tool_has_docs_and_code() {
        let missing: Vec<String> = TOOLS
            .iter()
            .flat_map(|t| {
                let mut m = Vec::new();
                if doc(t.id).is_none_or(|d| d.trim().is_empty()) {
                    m.push(format!("{}/doc.md", t.id));
                }
                for lang in Lang::ALL {
                    if code(t.id, lang).is_none_or(|c| c.trim().is_empty()) {
                        m.push(format!("{}/{}", t.id, lang.file_name()));
                    }
                }
                m
            })
            .collect();
        assert!(missing.is_empty(), "missing content: {missing:?}");
        // No stray directories for tools that do not exist.
        for e in ENTRIES {
            assert!(TOOLS.iter().any(|t| t.id == e.id), "unknown tool {}", e.id);
        }
    }

    #[test]
    fn docs_have_every_section_in_order() {
        for e in ENTRIES {
            let Some(doc) = e.doc else { continue };
            let headings: Vec<&str> = doc
                .lines()
                .filter_map(|l| l.strip_prefix("## "))
                .map(str::trim)
                .collect();
            assert_eq!(headings, SECTIONS, "{}", e.id);
            let tool = TOOLS.iter().find(|t| t.id == e.id).unwrap();
            assert_eq!(
                doc.lines().next().map(str::trim),
                Some(format!("# {}", tool.name).as_str()),
                "{} title",
                e.id
            );
        }
    }

    #[test]
    fn code_is_ascii() {
        for e in ENTRIES {
            for (lang, code) in Lang::ALL.iter().zip([e.python, e.typescript, e.rust]) {
                if let Some(code) = code
                    && let Some((n, line)) = code.lines().enumerate().find(|(_, l)| !l.is_ascii())
                {
                    panic!("{}/{} line {}: {line}", e.id, lang.file_name(), n + 1);
                }
            }
        }
    }

    /// The README's gallery shows every tool: a picture under `docs/media/`
    /// (made by `tools/capture`) and a link to its reference doc.
    #[test]
    fn readme_has_a_gallery_card_for_every_tool() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
        let mut missing = Vec::new();
        for t in TOOLS {
            let doc = format!("assets/content/{}/doc.md", t.id);
            if !readme.contains(&doc) {
                missing.push(doc);
            }
            let media = ["png", "gif"]
                .map(|ext| format!("docs/media/{}.{ext}", t.id))
                .into_iter()
                .find(|m| readme.contains(m.as_str()));
            match media {
                Some(m) if root.join(&m).is_file() => {}
                Some(m) => missing.push(format!("{m} (file)")),
                None => missing.push(format!("docs/media/{}.png", t.id)),
            }
        }
        assert!(missing.is_empty(), "README gallery is missing: {missing:?}");
    }

    #[test]
    fn landing_page_links_every_tool() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let page = std::fs::read_to_string(root.join("web/index.html")).unwrap();
        let missing: Vec<_> = TOOLS
            .iter()
            .flat_map(|t| [format!("app/#{}\"", t.id), format!("media/{}.", t.id)])
            .filter(|needle| !page.contains(needle.as_str()))
            .collect();
        assert!(missing.is_empty(), "web/index.html is missing: {missing:?}");
    }
}
