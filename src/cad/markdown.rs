//! A small markdown renderer for the reference docs.
//!
//! Supports what the docs use: `#`–`###` headings, paragraphs, bullet and
//! numbered lists, fenced code, ```` ```math ```` blocks (one equation per
//! line, `_{sub}` and `^{sup}`), pipe tables, `>` notes, and inline
//! `**bold**`, `*italic*` and `` `code` ``.

use egui::text::LayoutJob;
use egui::{Align, Color32, FontId, Frame, Margin, RichText, Stroke, TextFormat};

use super::theme::{Chrome, mono};
use crate::formula::{MathText, Palette};

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, String),
    Paragraph(String),
    /// (nesting level, text)
    Bullets(Vec<(usize, String)>),
    Numbered(Vec<String>),
    Code(String),
    Math(Vec<String>),
    Table(Vec<String>, Vec<Vec<String>>),
    Note(String),
}

pub fn parse(src: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    let mut para: Vec<&str> = Vec::new();
    let end_para = |para: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if !para.is_empty() {
            blocks.push(Block::Paragraph(para.join(" ")));
            para.clear();
        }
    };
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            end_para(&mut para, &mut blocks);
            i += 1;
        } else if let Some(fence) = trimmed.strip_prefix("```") {
            end_para(&mut para, &mut blocks);
            let math = fence.trim() == "math";
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                body.push(lines[i]);
                i += 1;
            }
            i += 1;
            blocks.push(if math {
                Block::Math(
                    body.iter()
                        .map(|l| l.trim().to_owned())
                        .filter(|l| !l.is_empty())
                        .collect(),
                )
            } else {
                Block::Code(body.join("\n"))
            });
        } else if trimmed.starts_with('#') {
            end_para(&mut para, &mut blocks);
            let level = trimmed.chars().take_while(|&c| c == '#').count() as u8;
            blocks.push(Block::Heading(
                level.min(3),
                trimmed[level as usize..].trim().to_owned(),
            ));
            i += 1;
        } else if trimmed.starts_with('|') {
            end_para(&mut para, &mut blocks);
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].trim().starts_with('|') {
                let row = lines[i].trim();
                let cells: Vec<String> = row
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_owned())
                    .collect();
                let separator = cells
                    .iter()
                    .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'));
                if !separator {
                    rows.push(cells);
                }
                i += 1;
            }
            if !rows.is_empty() {
                let header = rows.remove(0);
                blocks.push(Block::Table(header, rows));
            }
        } else if let Some(rest) = bullet(line) {
            end_para(&mut para, &mut blocks);
            let mut items = vec![rest];
            i += 1;
            while i < lines.len() {
                if let Some(item) = bullet(lines[i]) {
                    items.push(item);
                } else if !lines[i].trim().is_empty()
                    && lines[i].starts_with("  ")
                    && !items.is_empty()
                {
                    // A continuation line of the previous item.
                    let last = items.last_mut().unwrap();
                    last.1.push(' ');
                    last.1.push_str(lines[i].trim());
                } else {
                    break;
                }
                i += 1;
            }
            blocks.push(Block::Bullets(items));
        } else if let Some(rest) = numbered(trimmed) {
            end_para(&mut para, &mut blocks);
            let mut items = vec![rest.to_owned()];
            i += 1;
            while i < lines.len() {
                if let Some(item) = numbered(lines[i].trim()) {
                    items.push(item.to_owned());
                } else if !lines[i].trim().is_empty() && lines[i].starts_with("  ") {
                    let last = items.last_mut().unwrap();
                    last.push(' ');
                    last.push_str(lines[i].trim());
                } else {
                    break;
                }
                i += 1;
            }
            blocks.push(Block::Numbered(items));
        } else if let Some(rest) = trimmed.strip_prefix('>') {
            end_para(&mut para, &mut blocks);
            let mut text = rest.trim().to_owned();
            i += 1;
            while i < lines.len() && lines[i].trim().starts_with('>') {
                text.push(' ');
                text.push_str(lines[i].trim()[1..].trim());
                i += 1;
            }
            blocks.push(Block::Note(text));
        } else {
            para.push(trimmed);
            i += 1;
        }
    }
    end_para(&mut para, &mut blocks);
    blocks
}

fn bullet(line: &str) -> Option<(usize, String)> {
    let indent = line.len() - line.trim_start().len();
    let t = line.trim_start();
    let rest = t.strip_prefix("- ").or_else(|| t.strip_prefix("* "))?;
    Some((indent / 2, rest.trim().to_owned()))
}

fn numbered(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    line[digits..].strip_prefix(". ")
}

/// The `##` headings, for a table of contents.
pub fn sections(blocks: &[Block]) -> Vec<&str> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Heading(2, t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

/// Inline formatting: `**bold**`, `*italic*`, `` `code` ``.
pub fn inline(
    text: &str,
    size: f32,
    color: Color32,
    strong: Color32,
    code: Color32,
    code_bg: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base = TextFormat {
        font_id: FontId::proportional(size),
        color,
        ..Default::default()
    };
    let (mut bold, mut italic) = (false, false);
    let mut rest = text;
    let mut buf = String::new();
    let flush = |job: &mut LayoutJob, buf: &mut String, bold: bool, italic: bool| {
        if !buf.is_empty() {
            let mut f = base.clone();
            if bold {
                f.color = strong;
            }
            f.italics = italic;
            job.append(buf, 0.0, f);
            buf.clear();
        }
    };
    while let Some(c) = rest.chars().next() {
        if let Some(r) = rest.strip_prefix("**") {
            flush(&mut job, &mut buf, bold, italic);
            bold = !bold;
            rest = r;
        } else if c == '`'
            && let Some(end) = rest[1..].find('`')
        {
            flush(&mut job, &mut buf, bold, italic);
            job.append(
                &rest[1..1 + end],
                0.0,
                TextFormat {
                    font_id: mono(size * 0.92),
                    color: code,
                    background: code_bg,
                    ..Default::default()
                },
            );
            rest = &rest[end + 2..];
        } else if c == '*' && (italic || rest[1..].contains('*')) {
            flush(&mut job, &mut buf, bold, italic);
            italic = !italic;
            rest = &rest[1..];
        } else if c == '\\' && rest.len() > 1 {
            let next = rest[1..].chars().next().unwrap();
            buf.push(next);
            rest = &rest[1 + next.len_utf8()..];
        } else {
            buf.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    flush(&mut job, &mut buf, bold, italic);
    job
}

/// One equation line, with `_{...}` and `^{...}` set as sub- and superscripts.
pub fn math(line: &str) -> MathText {
    let mut m = MathText::new();
    let mut rest = line;
    let mut plain = String::new();
    while !rest.is_empty() {
        let script = if rest.starts_with("_{") {
            Some(false)
        } else if rest.starts_with("^{") {
            Some(true)
        } else {
            None
        };
        if let Some(sup) = script
            && let Some(end) = rest[2..].find('}')
        {
            if !plain.is_empty() {
                m = m.t(&plain);
                plain.clear();
            }
            let inner = &rest[2..2 + end];
            m = if sup { m.sup(inner) } else { m.sub(inner) };
            rest = &rest[3 + end..];
        } else {
            let c = rest.chars().next().unwrap();
            plain.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    if !plain.is_empty() {
        m = m.t(&plain);
    }
    m
}

/// Render parsed blocks. When `jump` names a section, scroll to it.
pub fn show(
    ui: &mut egui::Ui,
    blocks: &[Block],
    chrome: &Chrome,
    palette: &Palette,
    jump: Option<&str>,
) {
    let body = 13.5;
    let text = chrome.text.gamma_multiply(0.92);
    let strong = chrome.text;
    let code = chrome.dimension;
    let code_bg = chrome.well;
    let para = |ui: &mut egui::Ui, s: &str| {
        ui.label(inline(s, body, text, strong, code, code_bg));
    };
    for block in blocks {
        match block {
            Block::Heading(1, t) => {
                ui.add_space(4.0);
                ui.label(RichText::new(t).size(22.0).color(strong));
                ui.add_space(2.0);
            }
            Block::Heading(2, t) => {
                ui.add_space(14.0);
                let r = ui.label(
                    RichText::new(t.to_uppercase())
                        .size(12.0)
                        .strong()
                        .color(chrome.accent),
                );
                let y = r.rect.bottom() + 3.0;
                ui.painter().line_segment(
                    [
                        egui::pos2(r.rect.left(), y),
                        egui::pos2(ui.max_rect().right(), y),
                    ],
                    Stroke::new(1.0, chrome.border),
                );
                ui.add_space(6.0);
                if jump == Some(t.as_str()) {
                    r.scroll_to_me(Some(Align::TOP));
                }
            }
            Block::Heading(_, t) => {
                ui.add_space(6.0);
                ui.label(RichText::new(t).size(14.0).color(strong));
            }
            Block::Paragraph(s) => {
                para(ui, s);
                ui.add_space(6.0);
            }
            Block::Bullets(items) => {
                for (level, s) in items {
                    ui.horizontal_top(|ui| {
                        ui.add_space(4.0 + 14.0 * *level as f32);
                        ui.label(
                            RichText::new(if *level == 0 { "•" } else { "–" }).color(chrome.accent),
                        );
                        ui.add_space(2.0);
                        para(ui, s);
                    });
                }
                ui.add_space(6.0);
            }
            Block::Numbered(items) => {
                for (k, s) in items.iter().enumerate() {
                    ui.horizontal_top(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("{}.", k + 1))
                                .font(mono(12.0))
                                .color(chrome.accent),
                        );
                        para(ui, s);
                    });
                }
                ui.add_space(6.0);
            }
            Block::Code(s) => {
                Frame::new()
                    .fill(chrome.well)
                    .stroke(Stroke::new(1.0, chrome.border))
                    .inner_margin(Margin::same(8))
                    .corner_radius(3)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::ScrollArea::horizontal()
                            .id_salt(s.len())
                            .show(ui, |ui| {
                                ui.label(RichText::new(s).font(mono(12.0)).color(text));
                            });
                    });
                ui.add_space(6.0);
            }
            Block::Math(lines) => {
                Frame::new()
                    .fill(chrome.well)
                    .stroke(Stroke::new(1.0, chrome.border))
                    .inner_margin(Margin::symmetric(12, 8))
                    .corner_radius(3)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for l in lines {
                            let mut job = math(l).layout(palette, 16.0);
                            for s in &mut job.sections {
                                s.format.color = strong;
                            }
                            ui.label(job);
                        }
                    });
                ui.add_space(6.0);
            }
            Block::Table(header, rows) => {
                Frame::new()
                    .stroke(Stroke::new(1.0, chrome.border))
                    .inner_margin(Margin::same(6))
                    .corner_radius(3)
                    .show(ui, |ui| {
                        egui::Grid::new(("md-table", header.join("|")))
                            .striped(true)
                            .spacing([14.0, 4.0])
                            .show(ui, |ui| {
                                for h in header {
                                    ui.label(inline(
                                        h,
                                        12.0,
                                        chrome.accent,
                                        chrome.accent,
                                        code,
                                        code_bg,
                                    ));
                                }
                                ui.end_row();
                                for row in rows {
                                    for cell in row {
                                        ui.label(inline(cell, 12.5, text, strong, code, code_bg));
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(6.0);
            }
            Block::Note(s) => {
                let r = Frame::new()
                    .fill(chrome.accent.gamma_multiply(0.08))
                    .inner_margin(Margin {
                        left: 12,
                        right: 8,
                        top: 6,
                        bottom: 6,
                    })
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        para(ui, s);
                    });
                let rect = r.response.rect;
                ui.painter().line_segment(
                    [rect.left_top(), rect.left_bottom()],
                    Stroke::new(3.0, chrome.accent),
                );
                ui.add_space(6.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_block_kind() {
        let src = "# Title\n\n## Overview\n\nOne line\nsame paragraph.\n\n- a\n  - b\n- c\n  continued\n\n1. first\n2. second\n\n```math\nx_{n+1} = x_{n}^{2}\n```\n\n```\ncode here\n```\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\n> a note\n> more";
        let b = parse(src);
        assert_eq!(b[0], Block::Heading(1, "Title".into()));
        assert_eq!(b[1], Block::Heading(2, "Overview".into()));
        assert_eq!(b[2], Block::Paragraph("One line same paragraph.".into()));
        assert_eq!(
            b[3],
            Block::Bullets(vec![
                (0, "a".into()),
                (1, "b".into()),
                (0, "c continued".into())
            ])
        );
        assert_eq!(b[4], Block::Numbered(vec!["first".into(), "second".into()]));
        assert_eq!(b[5], Block::Math(vec!["x_{n+1} = x_{n}^{2}".into()]));
        assert_eq!(b[6], Block::Code("code here".into()));
        assert_eq!(
            b[7],
            Block::Table(
                vec!["A".into(), "B".into()],
                vec![vec!["1".into(), "2".into()]]
            )
        );
        assert_eq!(b[8], Block::Note("a note more".into()));
        assert_eq!(sections(&b), vec!["Overview"]);
    }

    #[test]
    fn math_scripts() {
        assert_eq!(math("x_{n+1} = x_{n}^{2} + c").text(), "xn+1 = xn2 + c");
    }

    #[test]
    fn inline_styles() {
        let job = inline(
            "a **b** *c* `d` \\*e",
            12.0,
            Color32::GRAY,
            Color32::WHITE,
            Color32::RED,
            Color32::BLACK,
        );
        assert_eq!(job.text, "a b c d *e");
        let colors: Vec<_> = job.sections.iter().map(|s| s.format.color).collect();
        assert!(colors.contains(&Color32::WHITE) && colors.contains(&Color32::RED));
        assert!(job.sections.iter().any(|s| s.format.italics));
    }

    #[test]
    fn every_doc_parses_into_its_sections() {
        for e in crate::content::ENTRIES {
            if let Some(doc) = e.doc {
                let blocks = parse(doc);
                assert_eq!(sections(&blocks), crate::content::SECTIONS, "{}", e.id);
            }
        }
    }
}
