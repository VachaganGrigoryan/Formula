//! The `.fwb` sheet format: plain text in CAD-style sections, like DXF.
//!
//! ```text
//! FWB 1
//! ; a comment
//! SECTION CAMERA
//!   center 0 0
//!   zoom 1
//! ENDSEC
//! ```
//!
//! The first line names the format and its version. After it come sections,
//! `SECTION <NAME>` to `ENDSEC`, each holding `key value` lines. Indentation is
//! cosmetic, blank lines and `;` lines are skipped. A value is written by
//! [`encode`]: one JSON value, or for a list of two or more numbers or
//! booleans, the items separated by spaces (`rect 0 0 640 480`).
//!
//! This module knows nothing about what the sections mean; `cad::sheet` maps
//! them to the workspace.

use std::fmt;

use serde_json::Value;

/// The format version this build writes, and the newest it reads.
pub const VERSION: u32 = 1;
/// The file extension of sheets.
pub const SHEET_EXT: &str = "fwb";

#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub version: u32,
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub name: String,
    /// The line of `SECTION`, 1-based (0 for sections built in code).
    pub line: usize,
    pub entries: Vec<Entry>,
}

impl Section {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            line: 0,
            entries: Vec::new(),
        }
    }

    /// Append `key value`.
    pub fn push(&mut self, key: &str, value: &Value) {
        self.entries.push(Entry {
            key: key.to_owned(),
            rest: encode(value),
            line: 0,
        });
    }

    /// Append `key name value`, e.g. `param a 3`.
    pub fn push_named(&mut self, key: &str, name: &str, value: &Value) {
        self.entries.push(Entry {
            key: key.to_owned(),
            rest: format!("{name} {}", encode(value)),
            line: 0,
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    /// The raw text after the key.
    pub rest: String,
    /// 1-based (0 for entries built in code).
    pub line: usize,
}

impl Entry {
    pub fn value(&self) -> Value {
        decode(&self.rest)
    }

    /// For `key name value` lines: the name and the value.
    pub fn split_name(&self) -> Option<(&str, Value)> {
        let (name, value) = self
            .rest
            .split_once(char::is_whitespace)
            .unwrap_or((&self.rest, ""));
        is_key(name).then(|| (name, decode(value)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FwbError {
    /// Not a sheet at all: empty, binary, or without the `FWB` line.
    NotASheet,
    /// Written by a newer version of the app.
    NewerVersion(u32),
    Malformed {
        line: usize,
        message: String,
    },
}

impl fmt::Display for FwbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FwbError::NotASheet => write!(f, "not a Formula sheet (no FWB line)"),
            FwbError::NewerVersion(v) => write!(
                f,
                "made by a newer version of the app (format {v}, this one reads up to {VERSION})"
            ),
            FwbError::Malformed { line, message } => write!(f, "line {line}: {message}"),
        }
    }
}

impl std::error::Error for FwbError {}

/// A key or name: lowercase letters, digits and `_`, not starting with a digit.
pub fn is_key(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_section_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_uppercase() || c == '_')
}

/// A value as written in a sheet: a list of two or more numbers or booleans
/// as its items separated by spaces, anything else as compact JSON.
pub fn encode(v: &Value) -> String {
    match v {
        Value::Array(items)
            if items.len() >= 2 && items.iter().all(|i| i.is_number() || i.is_boolean()) =>
        {
            items
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        }
        _ => v.to_string(),
    }
}

/// Read what [`encode`] wrote: one JSON value, else space-separated JSON
/// numbers or booleans as a list, else the text itself as a string.
pub fn decode(s: &str) -> Value {
    let s = s.trim();
    if let Ok(v) = serde_json::from_str::<Value>(s) {
        return v;
    }
    let items: Option<Vec<Value>> = s
        .split_whitespace()
        .map(|t| {
            serde_json::from_str::<Value>(t)
                .ok()
                .filter(|v| v.is_number() || v.is_boolean() || v.is_null())
        })
        .collect();
    match items {
        Some(items) if items.len() >= 2 => Value::Array(items),
        _ => Value::String(s.to_owned()),
    }
}

/// The sheet as text: the `FWB` line, then each section with its entries
/// indented by two spaces.
pub fn write(doc: &Doc) -> String {
    let mut out = format!("FWB {}\n; Formula Workbench sheet\n", doc.version);
    for s in &doc.sections {
        out.push_str(&format!("\nSECTION {}\n", s.name));
        for e in &s.entries {
            if e.rest.is_empty() {
                out.push_str(&format!("  {}\n", e.key));
            } else {
                out.push_str(&format!("  {} {}\n", e.key, e.rest));
            }
        }
        out.push_str("ENDSEC\n");
    }
    out
}

/// Parse a sheet. Anything that is not well-formed is an error naming the
/// line; what the sections and keys mean is checked by the caller.
pub fn read(text: &str) -> Result<Doc, FwbError> {
    if text.contains('\0') {
        return Err(FwbError::NotASheet);
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with(';'));

    let (_, header) = lines.next().ok_or(FwbError::NotASheet)?;
    let version = match header.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["FWB", v] => v.parse::<u32>().map_err(|_| FwbError::NotASheet)?,
        _ => return Err(FwbError::NotASheet),
    };
    if version == 0 {
        return Err(FwbError::NotASheet);
    }
    if version > VERSION {
        return Err(FwbError::NewerVersion(version));
    }

    let malformed = |line: usize, message: String| FwbError::Malformed { line, message };
    let mut sections = Vec::new();
    let mut open: Option<Section> = None;
    for (n, line) in lines {
        let (word, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match (word, open.as_mut()) {
            ("SECTION", Some(s)) => {
                return Err(malformed(
                    n,
                    format!(
                        "SECTION {rest} starts inside SECTION {} (line {}), which is missing ENDSEC",
                        s.name, s.line
                    ),
                ));
            }
            ("SECTION", None) => {
                if !is_section_name(rest) {
                    return Err(malformed(n, format!("bad section name \"{rest}\"")));
                }
                open = Some(Section {
                    name: rest.to_owned(),
                    line: n,
                    entries: Vec::new(),
                });
            }
            ("ENDSEC", _) => match open.take() {
                Some(s) => sections.push(s),
                None => return Err(malformed(n, "ENDSEC without a SECTION".into())),
            },
            (key, Some(s)) => {
                if !is_key(key) {
                    return Err(malformed(n, format!("bad key \"{key}\"")));
                }
                s.entries.push(Entry {
                    key: key.to_owned(),
                    rest: rest.to_owned(),
                    line: n,
                });
            }
            (_, None) => {
                return Err(malformed(n, format!("\"{line}\" is outside any SECTION")));
            }
        }
    }
    if let Some(s) = open {
        return Err(malformed(
            s.line,
            format!("SECTION {} is missing ENDSEC", s.name),
        ));
    }
    Ok(Doc { version, sections })
}

/// The example sheet from the roadmap, shared by the codec and sheet tests.
#[cfg(test)]
pub const EXAMPLE: &str = r#"FWB 1
; comment
SECTION HEADER
  app "0.1.0"
ENDSEC
SECTION CAMERA
  center 0 0
  zoom 1
ENDSEC
SECTION SETTINGS
  grid true
  snap true
  ortho false
  rulers true
  minimap true
  animate_all false
ENDSEC
SECTION UI
  theme dark
  browser true
  inspector true
  tab properties
  lang python
ENDSEC
SECTION VIEWPORT
  uid 3
  tool lissajous
  title "VP-03"
  rect 0 0 640 480
  locked false
  hidden false
  param a 3
  param points [[0,0],[1.5,2]]
ENDSEC
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn values_encode_compactly_and_decode_back() {
        let cases = [
            (json!([0, 0]), "0 0"),
            (json!([3]), "[3]"),
            (json!([]), "[]"),
            (json!([true, false]), "true false"),
            (json!(null), "null"),
            (json!(1.5), "1.5"),
            (json!("a \"b\" c"), r#""a \"b\" c""#),
            (json!([[0, 0], [1.5, 2]]), "[[0,0],[1.5,2]]"),
            (json!(["a b", "c"]), r#"["a b","c"]"#),
            (json!({"k": [1, 2]}), r#"{"k":[1,2]}"#),
        ];
        for (v, text) in cases {
            assert_eq!(encode(&v), text);
            assert_eq!(decode(text), v, "{text}");
        }
        // Unquoted words read as a string, so hand-written sheets stay lenient.
        assert_eq!(decode("dark"), json!("dark"));
        assert_eq!(decode("  2   3 "), json!([2, 3]));
    }

    /// A random JSON value, a few levels deep.
    fn random_value(rng: &mut crate::rng::Rng, depth: u32) -> Value {
        match rng.index(if depth == 0 { 4 } else { 6 }) {
            0 => json!(null),
            1 => json!(rng.f32() < 0.5),
            2 => json!((rng.range(-1e4, 1e4) * 100.0).round() / 100.0),
            3 => {
                let chars = ['a', ' ', '"', '\\', 'ж', '0', ';', '#'];
                let s: String = (0..rng.index(6))
                    .map(|_| chars[rng.index(chars.len())])
                    .collect();
                json!(s)
            }
            4 => Value::Array(
                (0..rng.index(5))
                    .map(|_| random_value(rng, depth - 1))
                    .collect(),
            ),
            _ => Value::Object(
                (0..rng.index(4))
                    .map(|k| (format!("k{k}"), random_value(rng, depth - 1)))
                    .collect(),
            ),
        }
    }

    #[test]
    fn random_values_round_trip() {
        let mut rng = crate::rng::Rng::new(17);
        for _ in 0..200 {
            let v = random_value(&mut rng, 3);
            assert_eq!(decode(&encode(&v)), v, "{}", encode(&v));
        }
    }

    #[test]
    fn the_example_reads_and_writes_back() {
        let doc = read(EXAMPLE).unwrap();
        assert_eq!(doc.version, 1);
        let names: Vec<&str> = doc.sections.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["HEADER", "CAMERA", "SETTINGS", "UI", "VIEWPORT"]);
        let vp = &doc.sections[4];
        assert_eq!(vp.line, 25);
        assert_eq!(vp.entries[3].key, "rect");
        assert_eq!(vp.entries[3].value(), json!([0, 0, 640, 480]));
        assert_eq!(vp.entries[3].line, 29);
        let (name, value) = vp.entries[7].split_name().unwrap();
        assert_eq!((name, value), ("points", json!([[0, 0], [1.5, 2]])));
        let again = read(&write(&doc)).unwrap();
        assert_eq!(write(&again), write(&doc));
        assert_eq!(again.sections.len(), 5);
    }

    #[test]
    fn comments_blank_lines_crlf_and_indentation_are_fine() {
        let text = "\u{feff}; first\r\n\r\nFWB 1\r\n   SECTION CAMERA\r\n\t; inside\r\n zoom 2\r\nENDSEC\r\n";
        let doc = read(text).unwrap();
        assert_eq!(doc.sections[0].entries[0].value(), json!(2));
        assert_eq!(read("FWB 1\n").unwrap().sections, Vec::new());
    }

    #[test]
    fn damaged_files_are_refused_with_the_line() {
        assert_eq!(read(""), Err(FwbError::NotASheet));
        assert_eq!(read("\0\u{1}garbage"), Err(FwbError::NotASheet));
        assert_eq!(read("SECTION A\nENDSEC\n"), Err(FwbError::NotASheet));
        assert_eq!(read("FWB x\n"), Err(FwbError::NotASheet));
        assert_eq!(read("FWB 99\n"), Err(FwbError::NewerVersion(99)));
        let line = |text: &str| match read(text) {
            Err(FwbError::Malformed { line, message }) => (line, message),
            other => panic!("{other:?}"),
        };
        let (n, m) = line("FWB 1\nSECTION VIEWPORT\n  uid 3\n");
        assert_eq!((n, m.as_str()), (2, "SECTION VIEWPORT is missing ENDSEC"));
        assert_eq!(line("FWB 1\nSECTION A\nSECTION B\nENDSEC\n").0, 3);
        assert_eq!(line("FWB 1\nENDSEC\n").0, 2);
        assert_eq!(line("FWB 1\nzoom 1\n").0, 2);
        assert_eq!(line("FWB 1\nSECTION A\n  A-b 1\nENDSEC\n").0, 3);
        assert_eq!(line("FWB 1\nSECTION a\nENDSEC\n").0, 2);
        // A file cut off in the middle of a section.
        let cut = &EXAMPLE[..EXAMPLE.len() - 10];
        assert_eq!(line(cut).0, 25);
        let msg = read("FWB 1\nSECTION VIEWPORT\n").unwrap_err().to_string();
        assert_eq!(msg, "line 2: SECTION VIEWPORT is missing ENDSEC");
        assert!(FwbError::NewerVersion(2).to_string().contains("newer"));
    }
}
