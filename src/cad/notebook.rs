//! Exporting a tool as a Jupyter notebook: its overview and formula, the
//! equations as LaTeX, and the Python visualization ready to run.

use serde_json::{Value, json};

/// Every exported script animates with matplotlib; ipympl makes that a live
/// widget inside the notebook.
const SETUP: &str = "%pip install -q numpy matplotlib ipympl\n%matplotlib widget\n";

/// An nbformat 4.5 notebook (`.ipynb` JSON).
pub fn notebook(doc_md: &str, latex: &str, python: &str) -> String {
    let nb = json!({
        "cells": [
            markdown("overview", overview(doc_md)),
            markdown("equations", &format!("$$\n{latex}\n$$")),
            code("setup", SETUP),
            code("visualization", python),
        ],
        "metadata": {
            "kernelspec": { "display_name": "Python 3", "language": "python", "name": "python3" },
            "language_info": { "name": "python" },
        },
        "nbformat": 4,
        "nbformat_minor": 5,
    });
    serde_json::to_string_pretty(&nb).unwrap_or_default()
}

/// The doc's title and its sections up to (not including) "Intuition".
fn overview(doc_md: &str) -> &str {
    doc_md
        .find("\n## Intuition")
        .map_or(doc_md, |end| &doc_md[..end])
}

fn markdown(id: &str, text: &str) -> Value {
    json!({ "cell_type": "markdown", "id": id, "metadata": {}, "source": lines(text) })
}

fn code(id: &str, text: &str) -> Value {
    json!({
        "cell_type": "code",
        "id": id,
        "metadata": {},
        "execution_count": null,
        "outputs": [],
        "source": lines(text),
    })
}

/// nbformat stores text as lines, each ending in a newline except the last.
fn lines(text: &str) -> Vec<&str> {
    let text = text.trim_end_matches('\n');
    text.split_inclusive('\n').collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{self, Lang};
    use crate::formulas::{TOOLS, ToolEnv};
    use serde_json::Value;

    const DOC: &str = "# Lerp\n\n## Overview\n\nBlend.\n\n## Formula\n\nv = a + t(b - a)\n\n## Intuition\n\nSlide.\n";

    fn parse(nb: &str) -> Value {
        serde_json::from_str(nb).expect("valid JSON")
    }

    fn source(cell: &Value) -> String {
        cell["source"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect()
    }

    #[test]
    fn notebook_has_overview_equations_setup_and_code_cells() {
        let nb = parse(&notebook(DOC, r"x = \theta", "import numpy\nprint(1)\n"));
        assert_eq!(nb["nbformat"], 4);
        assert_eq!(nb["nbformat_minor"], 5);
        assert_eq!(nb["metadata"]["kernelspec"]["name"], "python3");
        let cells = nb["cells"].as_array().unwrap();
        let kinds: Vec<&str> = cells
            .iter()
            .map(|c| c["cell_type"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["markdown", "markdown", "code", "code"]);
        let mut ids: Vec<&str> = cells.iter().map(|c| c["id"].as_str().unwrap()).collect();
        ids.dedup();
        assert_eq!(ids.len(), 4, "cell ids are unique");
        for code in &cells[2..] {
            assert!(code["outputs"].as_array().unwrap().is_empty());
            assert!(code["execution_count"].is_null());
        }
    }

    #[test]
    fn overview_cell_stops_before_intuition() {
        let nb = parse(&notebook(DOC, "x", "pass\n"));
        let overview = source(&nb["cells"][0]);
        assert!(overview.starts_with("# Lerp"));
        assert!(overview.contains("## Formula\n\nv = a + t(b - a)"));
        assert!(!overview.contains("Intuition"), "{overview}");
    }

    #[test]
    fn equations_cell_is_display_math() {
        let nb = parse(&notebook(DOC, r"x = \theta", "pass\n"));
        assert_eq!(source(&nb["cells"][1]), "$$\nx = \\theta\n$$");
    }

    #[test]
    fn setup_cell_enables_live_matplotlib_animation() {
        let nb = parse(&notebook(DOC, "x", "pass\n"));
        let setup = source(&nb["cells"][2]);
        assert!(setup.contains("%pip install -q numpy matplotlib ipympl"));
        assert!(setup.contains("%matplotlib widget"));
    }

    /// Sources are stored as lines, each ending in a newline except the last.
    #[test]
    fn code_cell_holds_the_script_as_lines() {
        let nb = parse(&notebook(DOC, "x", "a = 1\nb = 2\n"));
        let lines = nb["cells"][3]["source"].as_array().unwrap();
        assert_eq!(lines, &[Value::from("a = 1\n"), Value::from("b = 2")]);
    }

    #[test]
    fn every_tool_exports_a_valid_notebook() {
        let env = ToolEnv::new();
        for tool in TOOLS {
            let doc = content::doc(tool.id).unwrap();
            let python = content::code(tool.id, Lang::Python).unwrap();
            let latex = crate::latex::latex_snippet(&*(tool.make)(&env));
            let nb = parse(&notebook(doc, &latex, python));
            assert_eq!(
                source(&nb["cells"][3]),
                python.trim_end_matches('\n'),
                "{}",
                tool.id
            );
        }
    }

    /// The setup cell relies on every script showing its figure with
    /// `plt.show()`, which ipympl turns into a live widget.
    #[test]
    fn every_python_export_shows_its_figure() {
        for tool in TOOLS {
            let python = content::code(tool.id, Lang::Python).unwrap();
            assert!(python.contains("plt.show()"), "{}/main.py", tool.id);
        }
    }
}
