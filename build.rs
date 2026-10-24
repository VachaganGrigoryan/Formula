//! Embeds the per-tool reference docs and exportable code.
//!
//! Every directory `assets/content/<tool id>/` may hold `doc.md`, `main.py`,
//! `main.ts` and `main.rs`. This writes `$OUT_DIR/content.rs`, a table with an
//! `include_str!` for each file that exists, so adding content needs no code
//! changes. `content::tests` checks that every tool has all four files.
//!
//! It also embeds the example sheets, `assets/sheets/*.fwb`, as
//! `$OUT_DIR/sheets.rs` (File > Examples).
//!
//! On Windows it embeds `assets/icon/formula.ico` in `formula.exe`.

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

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("content.rs");
    std::fs::write(dest, out).expect("write content table");

    write_sheets(Path::new(&out_dir));

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/icon/formula.ico");
        winresource::WindowsResource::new()
            .set_icon("assets/icon/formula.ico")
            .compile_for("formula")
            .expect("embed the Windows icon");
    }
}

/// `SHEETS`: (file name, text) for each example sheet, sorted by name.
fn write_sheets(out_dir: &Path) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sheets");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files: Vec<_> = std::fs::read_dir(&root)
        .map(|it| {
            it.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "fwb"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    let mut out = String::from("pub static SHEETS: &[(&str, &str)] = &[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let _ = writeln!(
            out,
            "    ({name:?}, include_str!({:?})),",
            path.display().to_string()
        );
    }
    out.push_str("];\n");
    std::fs::write(out_dir.join("sheets.rs"), out).expect("write sheets table");
}
