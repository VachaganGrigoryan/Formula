//! A whole sheet, the board as saved in a `.fwb` file: the viewports with
//! their formulas' settings, the data they share, the camera, the canvas
//! settings and the panels.
//!
//! [`to_text`] and [`from_text`] map a [`Sheet`] to and from the sections of
//! [`fwb`]. Reading is lenient: an unknown section, key, tool or parameter is
//! skipped with a warning, and a missing one keeps its default, so a sheet
//! from a newer build with more settings still opens.

use egui::pos2;
use serde_json::{Map, Value, json};

use super::board::{DEFAULT_WIDTH, ViewportModel};
use super::camera::{Camera, MAX_ZOOM, MIN_ZOOM};
use super::canvas::Settings;
use super::fwb::{self, Doc, Entry, FwbError, Section};
use super::inspector::Tab;
use crate::content::Lang;
use crate::formulas::{self, ToolEnv};

// `SHEETS`: the example sheets in `assets/sheets/`, embedded by build.rs.
include!(concat!(env!("OUT_DIR"), "/sheets.rs"));

/// The example sheets as (title, text), for File > Examples. The title is
/// the sheet's own, else its file name.
pub fn examples() -> Vec<(String, &'static str)> {
    SHEETS
        .iter()
        .map(|(file, text)| {
            let title = from_text(text)
                .ok()
                .and_then(|(s, _)| s.title)
                .unwrap_or_else(|| (*file).to_owned());
            (title, *text)
        })
        .collect()
}

/// The panels around the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiState {
    pub dark: bool,
    pub browser: bool,
    pub inspector: bool,
    pub tab: Tab,
    pub lang: Lang,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            dark: true,
            browser: true,
            inspector: true,
            tab: Tab::Properties,
            lang: Lang::Python,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Sheet {
    pub title: Option<String>,
    pub camera: Camera,
    pub settings: Settings,
    pub ui: UiState,
    /// The viewports' shared data ([`ToolEnv::save_state`]).
    pub shared: Map<String, Value>,
    /// Bottom to top.
    pub viewports: Vec<ViewportModel>,
}

const TABS: [(Tab, &str); 4] = [
    (Tab::Properties, "properties"),
    (Tab::Docs, "docs"),
    (Tab::Code, "code"),
    (Tab::Derive, "derive"),
];
const LANGS: [(Lang, &str); 3] = [
    (Lang::Python, "python"),
    (Lang::TypeScript, "typescript"),
    (Lang::Rust, "rust"),
];

fn name_of<T: PartialEq + Copy>(table: &[(T, &'static str)], v: T) -> &'static str {
    table.iter().find(|(t, _)| *t == v).map_or("", |(_, n)| n)
}

fn from_name<T: Copy>(table: &[(T, &'static str)], name: &str) -> Option<T> {
    table.iter().find(|(_, n)| *n == name).map(|(t, _)| *t)
}

/// The sheet as `.fwb` text.
pub fn to_text(sheet: &Sheet) -> String {
    let mut doc = Doc {
        version: fwb::VERSION,
        sections: Vec::new(),
    };

    let mut header = Section::new("HEADER");
    header.push("app", &json!(env!("CARGO_PKG_VERSION")));
    if let Some(title) = &sheet.title {
        header.push("title", &json!(title));
    }
    doc.sections.push(header);

    let mut camera = Section::new("CAMERA");
    camera.push(
        "center",
        &short(&[sheet.camera.center.x, sheet.camera.center.y]),
    );
    camera.push("zoom", &short(&sheet.camera.zoom));
    doc.sections.push(camera);

    let mut settings = Section::new("SETTINGS");
    if let Some(Value::Object(fields)) = crate::state::to_value(&sheet.settings) {
        for (k, v) in &fields {
            settings.push(k, v);
        }
    }
    doc.sections.push(settings);

    let ui = &sheet.ui;
    let mut s = Section::new("UI");
    s.push("theme", &json!(if ui.dark { "dark" } else { "light" }));
    s.push("browser", &json!(ui.browser));
    s.push("inspector", &json!(ui.inspector));
    s.push("tab", &json!(name_of(&TABS, ui.tab)));
    s.push("lang", &json!(name_of(&LANGS, ui.lang)));
    doc.sections.push(s);

    // Only shared data that differs from a new board's.
    let defaults = ToolEnv::new().save_state();
    let changed: Vec<_> = sheet
        .shared
        .iter()
        .filter(|(k, v)| defaults.get(*k) != Some(*v))
        .collect();
    if !changed.is_empty() {
        let mut shared = Section::new("SHARED");
        for (k, v) in changed {
            shared.push(k, v);
        }
        doc.sections.push(shared);
    }

    for m in &sheet.viewports {
        let mut vp = Section::new("VIEWPORT");
        vp.push("uid", &json!(m.uid));
        vp.push("tool", &json!(m.tool));
        vp.push("title", &json!(m.title));
        vp.push("rect", &short(&m.rect));
        vp.push("locked", &json!(m.locked));
        vp.push("hidden", &json!(m.hidden));
        if let Some(Value::Object(state)) = &m.state {
            for (name, v) in state {
                vp.push_named("param", name, v);
            }
        }
        doc.sections.push(vp);
    }
    fwb::write(&doc)
}

/// Read a sheet, with a warning for everything that was skipped. Errors only
/// for text that is not a well-formed sheet (see [`fwb::read`]).
pub fn from_text(text: &str) -> Result<(Sheet, Vec<String>), FwbError> {
    let doc = migrate(fwb::read(text)?);
    let mut sheet = Sheet::default();
    // Shared data the sheet leaves out is a new board's.
    let shared_keys = ToolEnv::new().save_state();
    sheet.shared = shared_keys.clone();
    let mut warnings = Vec::new();

    for section in &doc.sections {
        match section.name.as_str() {
            "HEADER" => {
                for e in &section.entries {
                    match (e.key.as_str(), e.value()) {
                        ("app", _) => {}
                        ("title", Value::String(t)) => sheet.title = Some(t),
                        ("title", _) => warn(&mut warnings, e, "the title is not text"),
                        (k, _) => warn(&mut warnings, e, &format!("unknown key \"{k}\" in HEADER")),
                    }
                }
            }
            "CAMERA" => {
                for e in &section.entries {
                    let v = e.value();
                    match e.key.as_str() {
                        "center" => match numbers::<2>(&v) {
                            Some([x, y]) => sheet.camera.center = pos2(x, y),
                            None => warn(&mut warnings, e, "center needs two numbers"),
                        },
                        "zoom" => match v.as_f64().filter(|z| z.is_finite()) {
                            Some(z) => sheet.camera.zoom = (z as f32).clamp(MIN_ZOOM, MAX_ZOOM),
                            None => warn(&mut warnings, e, "zoom needs a number"),
                        },
                        k => warn(&mut warnings, e, &format!("unknown key \"{k}\" in CAMERA")),
                    }
                }
            }
            "SETTINGS" => {
                for e in &section.entries {
                    let field = Value::Object(Map::from_iter([(e.key.clone(), e.value())]));
                    let merged = crate::state::merge(&sheet.settings, &field);
                    let known = crate::state::to_value(&sheet.settings)
                        .is_some_and(|v| v.get(&e.key).is_some());
                    if !known {
                        warn(
                            &mut warnings,
                            e,
                            &format!("unknown key \"{}\" in SETTINGS", e.key),
                        );
                    } else if crate::state::to_value(&merged).and_then(|v| v.get(&e.key).cloned())
                        != Some(e.value())
                    {
                        warn(&mut warnings, e, &format!("{} needs true or false", e.key));
                    }
                    sheet.settings = merged;
                }
            }
            "UI" => {
                for e in &section.entries {
                    let v = e.value();
                    let ui = &mut sheet.ui;
                    match (e.key.as_str(), v.as_str(), v.as_bool()) {
                        ("theme", Some("dark"), _) => ui.dark = true,
                        ("theme", Some("light"), _) => ui.dark = false,
                        ("browser", _, Some(b)) => ui.browser = b,
                        ("inspector", _, Some(b)) => ui.inspector = b,
                        ("tab", Some(name), _) if from_name(&TABS, name).is_some() => {
                            ui.tab = from_name(&TABS, name).unwrap();
                        }
                        ("lang", Some(name), _) if from_name(&LANGS, name).is_some() => {
                            ui.lang = from_name(&LANGS, name).unwrap();
                        }
                        ("theme" | "browser" | "inspector" | "tab" | "lang", _, _) => {
                            warn(
                                &mut warnings,
                                e,
                                &format!("bad value for {}: {}", e.key, e.rest),
                            );
                        }
                        (k, _, _) => warn(&mut warnings, e, &format!("unknown key \"{k}\" in UI")),
                    }
                }
            }
            "SHARED" => {
                for e in &section.entries {
                    if shared_keys.contains_key(&e.key) {
                        sheet.shared.insert(e.key.clone(), e.value());
                    } else {
                        warn(
                            &mut warnings,
                            e,
                            &format!("unknown key \"{}\" in SHARED", e.key),
                        );
                    }
                }
            }
            "VIEWPORT" => {
                let index = sheet.viewports.len();
                if let Some(m) = viewport(section, index, &mut warnings) {
                    sheet.viewports.push(m);
                }
            }
            name => warnings.push(format!(
                "line {}: unknown section {name}, skipped",
                section.line
            )),
        }
    }
    Ok((sheet, warnings))
}

/// One VIEWPORT section as a model, or `None` (with a warning) when its tool
/// is missing or unknown.
fn viewport(section: &Section, index: usize, warnings: &mut Vec<String>) -> Option<ViewportModel> {
    let mut m = ViewportModel {
        uid: index as u64 + 1,
        tool: String::new(),
        rect: [0.0, 0.0, DEFAULT_WIDTH, DEFAULT_WIDTH * 0.75],
        title: String::new(),
        locked: false,
        hidden: false,
        state: None,
    };
    let mut params: Vec<(&Entry, String, Value)> = Vec::new();
    for e in &section.entries {
        let v = e.value();
        match e.key.as_str() {
            "uid" => match v.as_u64() {
                Some(u) => m.uid = u,
                None => warn(warnings, e, "uid needs a whole number"),
            },
            "tool" => match v.as_str() {
                Some(t) => m.tool = t.to_owned(),
                None => warn(warnings, e, "tool needs a name"),
            },
            "title" => match v.as_str() {
                Some(t) => m.title = t.to_owned(),
                None => warn(warnings, e, "the title is not text"),
            },
            "rect" => match numbers::<4>(&v) {
                Some(r) => m.rect = r,
                None => warn(warnings, e, "rect needs four numbers: x y width height"),
            },
            "locked" | "hidden" => match v.as_bool() {
                Some(b) if e.key == "locked" => m.locked = b,
                Some(b) => m.hidden = b,
                None => warn(warnings, e, &format!("{} needs true or false", e.key)),
            },
            "param" => match e.split_name() {
                Some((name, value)) => params.push((e, name.to_owned(), value)),
                None => warn(warnings, e, "param needs a name and a value"),
            },
            k => warn(warnings, e, &format!("unknown key \"{k}\" in VIEWPORT")),
        }
    }
    if m.title.is_empty() {
        m.title = format!("VP-{:02}", m.uid);
    }
    let Some(tool) = formulas::tool(&m.tool) else {
        warnings.push(if m.tool.is_empty() {
            format!("line {}: skipped {} (no tool)", section.line, m.title)
        } else {
            format!(
                "line {}: skipped {} (unknown tool \"{}\")",
                section.line, m.title, m.tool
            )
        });
        return None;
    };
    if !params.is_empty() {
        let known = (tool.make)(&ToolEnv::new()).state_params();
        let mut state = Map::new();
        for (e, name, value) in params {
            if known.contains(&name) {
                state.insert(name, value);
            } else {
                warnings.push(format!(
                    "line {}: unknown param \"{name}\" for {}",
                    e.line, tool.id
                ));
            }
        }
        m.state = Some(Value::Object(state));
    }
    Some(m)
}

/// Note that the line of `e` was skipped or had a bad value.
fn warn(warnings: &mut Vec<String>, e: &Entry, what: &str) {
    warnings.push(format!("line {}: {what}", e.line));
}

/// `v` as JSON with `f32`s in their shortest form.
fn short<T: serde::Serialize>(v: &T) -> Value {
    crate::state::to_value(v).unwrap_or(Value::Null)
}

/// `N` finite numbers from a list.
fn numbers<const N: usize>(v: &Value) -> Option<[f32; N]> {
    let items = v.as_array()?;
    if items.len() != N {
        return None;
    }
    let mut out = [0.0; N];
    for (o, i) in out.iter_mut().zip(items) {
        *o = i.as_f64().filter(|n| n.is_finite())? as f32;
    }
    Some(out)
}

/// Bring a sheet written in an older format version up to [`fwb::VERSION`].
/// Version 1 is the first, so there is nothing to do yet.
fn migrate(doc: Doc) -> Doc {
    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cad::board::Board;

    /// A board with one viewport of every tool, each with its default state.
    fn every_tool_sheet() -> Sheet {
        let mut board = Board::default();
        for (i, tool) in formulas::TOOLS.iter().enumerate() {
            board.add(tool, pos2(i as f32 * 700.0, 0.0), egui::Vec2::ZERO, 0.0);
        }
        Sheet {
            title: Some("Everything \"at once\"".into()),
            viewports: board.to_model(),
            shared: board.env.save_state(),
            ..Sheet::default()
        }
    }

    #[test]
    fn a_sheet_with_every_tool_round_trips() {
        let mut sheet = every_tool_sheet();
        sheet.camera.center = pos2(12.5, -3.0);
        sheet.camera.zoom = 0.5;
        sheet.settings.ortho = true;
        sheet.ui = UiState {
            dark: false,
            browser: false,
            inspector: true,
            tab: Tab::Derive,
            lang: Lang::Rust,
        };
        sheet.viewports[0].locked = true;
        sheet.viewports[1].hidden = true;
        let text = to_text(&sheet);
        let (back, warnings) = from_text(&text).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(back, sheet);
    }

    #[test]
    fn the_example_parses_exactly() {
        let (sheet, warnings) = from_text(fwb::EXAMPLE).unwrap();
        // The roadmap's example gives Lissajous a `points` it does not have.
        assert_eq!(
            warnings,
            ["line 33: unknown param \"points\" for lissajous"]
        );
        assert_eq!(sheet.camera.center, pos2(0.0, 0.0));
        assert_eq!(sheet.camera.zoom, 1.0);
        assert!(!sheet.settings.animate_all);
        assert_eq!(sheet.ui, UiState::default());
        let vp = &sheet.viewports[0];
        assert_eq!(
            (vp.uid, vp.tool.as_str(), vp.title.as_str()),
            (3, "lissajous", "VP-03")
        );
        assert_eq!(vp.rect, [0.0, 0.0, 640.0, 480.0]);
        assert_eq!(vp.state, Some(json!({"a": 3})));
    }

    #[test]
    fn unknown_things_are_skipped_with_one_warning_each() {
        let text = format!(
            "{}\nSECTION LAYERS\n  color red\nENDSEC\nSECTION VIEWPORT\n  tool frobnicator\n  title \"VP-09\"\nENDSEC\nSECTION VIEWPORT\n  tool lerp\n  shadow true\n  param t 0.25\nENDSEC\n",
            fwb::EXAMPLE
        );
        let (sheet, warnings) = from_text(&text).unwrap();
        assert_eq!(warnings.len(), 4, "{warnings:?}");
        assert!(warnings[0].contains("unknown param \"points\" for lissajous"));
        assert!(warnings[1].contains("unknown section LAYERS"));
        assert!(warnings[2].contains("skipped VP-09 (unknown tool \"frobnicator\")"));
        assert!(warnings[3].contains("unknown key \"shadow\""));
        let tools: Vec<&str> = sheet.viewports.iter().map(|m| m.tool.as_str()).collect();
        assert_eq!(tools, ["lissajous", "lerp"]);
        assert_eq!(sheet.viewports[1].state, Some(json!({"t": 0.25})));
    }

    #[test]
    fn bad_values_keep_their_defaults() {
        let text = "FWB 1\nSECTION CAMERA\n  zoom 1e9\n  center 1\nENDSEC\nSECTION SETTINGS\n  grid maybe\nENDSEC\nSECTION UI\n  theme purple\n  tab code\nENDSEC\n";
        let (sheet, warnings) = from_text(text).unwrap();
        assert_eq!(sheet.camera.zoom, MAX_ZOOM);
        assert_eq!(sheet.settings, Settings::default());
        assert_eq!(sheet.ui.tab, Tab::Code);
        assert!(sheet.ui.dark);
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }

    #[test]
    fn every_example_opens_without_warnings() {
        assert_eq!(SHEETS.len(), 3);
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        let font = egui::FontId::proportional(14.0);
        for ((file, text), (title, _)) in SHEETS.iter().zip(examples()) {
            let (sheet, warnings) = from_text(text).unwrap();
            assert!(warnings.is_empty(), "{file}: {warnings:?}");
            assert!(!sheet.viewports.is_empty(), "{file}");
            assert_eq!(sheet.title.as_deref(), Some(title.as_str()), "{file}");
            for c in title.chars() {
                assert!(ctx.fonts_mut(|f| f.has_glyph(&font, c)), "{file}: {c:?}");
            }
        }
    }

    #[test]
    fn an_empty_sheet_is_valid() {
        let (sheet, warnings) = from_text("FWB 1\n").unwrap();
        assert!(warnings.is_empty());
        assert_eq!(
            sheet,
            Sheet {
                shared: ToolEnv::new().save_state(),
                ..Sheet::default()
            }
        );
    }
}
