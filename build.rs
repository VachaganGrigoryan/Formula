//! Embeds the per-tool reference docs and exportable code.
//!
//! Every directory `assets/content/<tool id>/` may hold `doc.md`, `main.py`,
//! `main.ts` and `main.rs`. This writes `$OUT_DIR/content.rs`, a table with an
//! `include_str!` for each file that exists, so adding content needs no code
//! changes. `content::tests` checks that every tool has all four files.

use std::fmt::Write as _;
use std::path::Path;

const FILES: [&str; 4] = ["doc.md", "main.py", "main.ts", "main.rs"];

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/content");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .map(|it| {
            it.filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();

    let mut out = String::from("pub static ENTRIES: &[Entry] = &[\n");
    for dir in dirs {
        let id = dir.file_name().unwrap().to_string_lossy().into_owned();
        let _ = write!(out, "    Entry {{\n        id: {id:?},\n");
        for (field, file) in ["doc", "python", "typescript", "rust"].iter().zip(FILES) {
            let path = dir.join(file);
            if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
                let _ = writeln!(
                    out,
                    "        {field}: Some(include_str!({:?})),",
                    path.display().to_string()
                );
            } else {
                let _ = writeln!(out, "        {field}: None,");
            }
        }
        out.push_str("    },\n");
    }
    out.push_str("];\n");

    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("content.rs");
    std::fs::write(dest, out).expect("write content table");
}
